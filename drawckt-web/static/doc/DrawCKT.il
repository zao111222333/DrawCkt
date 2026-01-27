/********************************************* 
schoa2json.il

Description: 
Exports Schematic/Symbol to JSON.
Key Feature: 
1. symbolTemplates is now a List/Array.
2. Symbol objects contain explicit "lib" and "cell" fields.
3. FillStyle is integer (0-5).
**********************************************/


/**********************************************
* Helper: Escape strings for JSON
**********************************************/
procedure( _jsonEscape( str )
    let( (out)
        if( stringp(str) then
            out = buildString(parseString(str "\\") "\\\\")
            out = buildString(parseString(out "\"") "\\\"")
        else
            out = ""
        )
        out
    )
)

/**********************************************
* Helper: Build Fill Style Lookup Table
**********************************************/
procedure( _buildFillTable( cv )
    let( (tf displayID fillTable packetName fillStyle layer purpose techLPP)
        
        fillTable = makeTable("fillTable" 1) ; Default 1 (Outline)
        
        tf = techGetTechFile(cv)
        displayID = drGetDisplay("display")
        
        if( tf && displayID then
            foreach( lpp cv~>lpps 
                layer = lpp~>layerName
                purpose = lpp~>purpose
                fillStyle = 1 
                
                techLPP = techGetLP(tf list(layer purpose))
                
                if( techLPP then
                    packetName = techGetLPPacketName(techLPP)
                    if( packetName then
                        fillStyle = drGetPacketFillStyle(displayID packetName)
                    )
                )
                fillTable[list(layer purpose)] = fillStyle
            )
        )
        fillTable
    )
)

/**********************************************
* Helper: Core Shape to JSON Converter
* Handles geometry formatting for both Symbol and Schematic shapes
**********************************************/
procedure( _shapeToJson( shape fillTable )
    let( (sType sLayer sPurp bbox fillVal jsonData)
        
        sType = shape~>objType
        sLayer = shape~>layerName
        sPurp = shape~>purpose
        
        ; Get Fill Style (Integer)
        fillVal = fillTable[ list(sLayer sPurp) ]
        if( null(fillVal) then fillVal = 1 )
        
        jsonData = nil

        case( sType
            (("line" "path")
                ; For graphic lines (non-wires), we output points
                sprintf(jsonData "    {\"type\": \"line\", \"layer\": \"%s\", \"points\": [%s]}" 
                    sLayer
                    buildString(
                        foreach(mapcar pt shape~>points sprintf(nil "[%L,%L]" xCoord(pt) yCoord(pt))) 
                        ", "
                    )
                )
            )
            ("rect"
                bbox = shape~>bBox
                sprintf(jsonData "    {\"type\": \"rect\", \"layer\": \"%s\", \"fillStyle\": %d, \"bBox\": [[%L,%L],[%L,%L]]}" 
                    sLayer fillVal
                    xCoord(lowerLeft(bbox)) yCoord(lowerLeft(bbox))
                    xCoord(upperRight(bbox)) yCoord(upperRight(bbox))
                )
            )
            ("ellipse" 
                bbox = shape~>bBox
                sprintf(jsonData "    {\"type\": \"ellipse\", \"layer\": \"%s\", \"fillStyle\": %d, \"bBox\": [[%L,%L],[%L,%L]]}" 
                    sLayer fillVal
                    xCoord(lowerLeft(bbox)) yCoord(lowerLeft(bbox))
                    xCoord(upperRight(bbox)) yCoord(upperRight(bbox))
                )
            )
            ("arc"
                bbox = shape~>ellipseBBox
                sprintf(jsonData "    {\"type\": \"arc\", \"layer\": \"%s\", \"fillStyle\": %d, \"bBox\": [[%L,%L],[%L,%L]], \"startAngle\": %L, \"stopAngle\": %L}" 
                    sLayer fillVal
                    xCoord(lowerLeft(bbox)) yCoord(lowerLeft(bbox))
                    xCoord(upperRight(bbox)) yCoord(upperRight(bbox))
                    shape~>startAngle shape~>stopAngle
                )
            )
            ("polygon"
                sprintf(jsonData "    {\"type\": \"polygon\", \"layer\": \"%s\", \"fillStyle\": %d, \"points\": [%s]}" 
                    sLayer fillVal
                    buildString(
                        foreach(mapcar pt shape~>points sprintf(nil "[%L,%L]" xCoord(pt) yCoord(pt))) 
                        ", "
                    )
                )
            )
            ("label"
                sprintf(jsonData "    {\"type\": \"label\", \"layer\": \"%s\", \"text\": \"%s\", \"xy\": [%L,%L], \"orient\": \"%s\", \"height\": %L, \"justify\": \"%s\", \"font\": \"%s\"}"
                    sLayer
                    _jsonEscape(shape~>theLabel)
                    xCoord(shape~>xy) yCoord(shape~>xy)
                    shape~>orient
                    shape~>height
                    shape~>justify
                    shape~>font
                )
            )
        )
        jsonData
    )
)

/**********************************************
* Helper: Extract Graphics from a Symbol View
**********************************************/
procedure( _extractSymbolGraphics( libName cellName )
let( (cv shapeList pinList shapeData pinData masterJson fillTable)
    
    cv = dbOpenCellViewByType(libName cellName "symbol" "schematicSymbol" "r")
    
    if( cv then
        fillTable = _buildFillTable(cv)
        
        ; 1. Extract Shapes using shared helper
        shapeList = nil
        foreach(shape cv~>shapes
            shapeData = _shapeToJson(shape fillTable)
            if( shapeData then
                shapeList = cons(shapeData shapeList)
            )
        )

        ; 2. Extract Pins
        pinList = nil
        foreach(term cv~>terminals
            let( (px py)
                if( term~>pins && car(term~>pins)~>fig then
                    px = xCoord(centerBox(car(term~>pins)~>fig~>bBox))
                    py = yCoord(centerBox(car(term~>pins)~>fig~>bBox))
                else px=0.0 py=0.0)
                sprintf(pinData "{\"name\": \"%s\", \"direction\": \"%s\", \"x\": %L, \"y\": %L}"
                    term~>name term~>direction px py
                )
                pinList = cons(pinData pinList)
            )
        )
        
        dbClose(cv)
        
        sprintf(masterJson 
            "    {\n      \"lib\": \"%s\",\n      \"cell\": \"%s\",\n      \"shapes\": [%s],\n      \"pins\": [%s]\n    }"
            libName cellName
            buildString(reverse(shapeList) ", ")
            buildString(reverse(pinList) ", ")
        )
    else
        _showErrorDialog(sprintf(nil "Could not open symbol view for %s/%s" libName cellName))
        masterJson = nil
    )
    masterJson
)
)

/**********************************************
* Main Function: schoa2json
**********************************************/
procedure( schoa2json(cv outputJsonPath)
let( (instList wireList pinList labelList schematicShapeList
      masterTable uniqueMasters masterJsonList
      outPort instData wireData pinData labelData shapeData 
      topFillTable sType)

    ; 0. Build Fill Table for Schematic level
    topFillTable = _buildFillTable(cv)

    ; 1. Instances
    instList = nil
    masterTable = makeTable("masters" nil)
    foreach(inst cv~>instances
        sprintf(instData 
            "    {\"name\": \"%s\", \"lib\": \"%s\", \"cell\": \"%s\", \"x\": %L, \"y\": %L, \"orient\": \"%s\"}" 
            inst~>name inst~>libName inst~>cellName 
            xCoord(inst~>xy) yCoord(inst~>xy) inst~>orient
        )
        instList = cons(instData instList)
        masterTable[list(inst~>libName inst~>cellName)] = t
    )

    ; 2. Process Shapes (Separating Wires, Labels, and Graphics)
    wireList = nil
    labelList = nil
    schematicShapeList = nil
    
    foreach(shape cv~>shapes
        sType = shape~>objType
        
        cond(
            ; Case A: Labels (Text)
            ( sType == "label" 
                shapeData = _shapeToJson(shape topFillTable)
                if(shapeData labelList = cons(shapeData labelList))
            )
            
            ; Case B: Wires (Lines/Paths with connectivity)
            ( ((sType == "line" || sType == "path") && shape~>net)
                let( (pts)
                    pts = foreach(mapcar pt shape~>points sprintf(nil "[%L,%L]" xCoord(pt) yCoord(pt)))
                    sprintf(wireData "    {\"net\": \"%s\", \"points\": [%s]}" 
                        shape~>net~>name || "UNNAMED" buildString(pts ", ")
                    )
                    wireList = cons(wireData wireList)
                )
            )
            
            ; Case C: Schematic Graphics (Rects, Circles, or Lines without Net)
            ( t ; Capture everything else that is a drawable shape
                ; Note: _shapeToJson filters supported types (rect, ellipse, etc.)
                ; It will return nil for types it doesn't know (like inst)
                shapeData = _shapeToJson(shape topFillTable)
                if( shapeData then
                    schematicShapeList = cons(shapeData schematicShapeList)
                )
            )
        )
    )

    ; 3. Top Pins
    pinList = nil
    foreach(term cv~>terminals
        let( (px py)
            if( term~>pins && car(term~>pins)~>fig then
                px = xCoord(car(term~>pins)~>fig~>xy)
                py = yCoord(car(term~>pins)~>fig~>xy)
            else px=0.0 py=0.0)
            sprintf(pinData "    {\"name\": \"%s\", \"direction\": \"%s\", \"x\": %L, \"y\": %L}" 
                term~>name term~>direction px py
            )
            pinList = cons(pinData pinList)
        )
    )

    ; 4. Symbol Templates
    masterJsonList = nil
    printf("Extracting symbol graphics for %d unique masters...\n" length(masterTable))
    foreach( key masterTable
        let( (lib cell mJson)
            lib = car(key) cell = cadr(key)
            mJson = _extractSymbolGraphics(lib cell)
            if( mJson then masterJsonList = cons(mJson masterJsonList))
        )
    )

    ; 5. Write File
    outPort = outfile(outputJsonPath)
    if( outPort then
        fprintf(outPort "{\n")
        fprintf(outPort "  \"design\": {\"lib\": \"%s\", \"cell\": \"%s\"},\n" cv~>libName cv~>cellName)
        
        fprintf(outPort "  \"instances\": [\n%s\n  ],\n" buildString(reverse(instList) ",\n"))
        fprintf(outPort "  \"wires\": [\n%s\n  ],\n" buildString(reverse(wireList) ",\n"))
        fprintf(outPort "  \"labels\": [\n%s\n  ],\n" buildString(reverse(labelList) ",\n"))
        fprintf(outPort "  \"shapes\": [\n%s\n  ],\n" buildString(reverse(schematicShapeList) ",\n")) 
        fprintf(outPort "  \"pins\": [\n%s\n  ],\n" buildString(reverse(pinList) ",\n"))
        fprintf(outPort "  \"symbols\": [\n%s\n  ]\n" buildString(masterJsonList ",\n"))
        
        fprintf(outPort "}\n")
        close(outPort)
        printf("Export Complete: %s\n" outputJsonPath)
        ; Return success status
        t
    else
        printf("Failed to open output file: %s\n" outputJsonPath)
        ; Return failure status
        nil
    )
)
)   


/**********************************************
* Helper: Display error dialog
**********************************************/
procedure( _showErrorDialog(message)
    hiDisplayAppDBox(
        ?name 'errorDBox
        ?dboxBanner "Error"
        ?buttonLayout 'Close
        ?dboxText message
    )
)

/**********************************************
* Helper: Display success dialog
**********************************************/
procedure( _showSuccessDialog(message)
    hiDisplayAppDBox(
        ?name 'successDBox
        ?dboxBanner "Success"
        ?buttonLayout 'Close
        ?dboxText message
    )
)

/**********************************************
* Helper: Remove quotes from string field value
* hiCreateStringField automatically adds quotes, so we need to remove them
**********************************************/
procedure( _removeQuotes(str)
    let( (len firstChar lastChar result)
        if( stringp(str) then
            len = strlen(str)
            if( len >= 2 then
                firstChar = substring(str 0 1)
                lastChar = substring(str len-1 1)
                if( firstChar == "\"" && lastChar == "\"" then
                    result = substring(str 1 len-2)
                else
                    result = str
                )
            else
                result = str
            )
        else
            result = str
        )
        result
    )
)

/**********************************************
* FUNCTION: exportSchematicToJson
* DESCRIPTION: 
* Main callback function. Gets the current schematic cellview,
* creates a dialog form for user to enter output JSON file path,
* and calls schoa2json.
**********************************************/
procedure( exportSchematicToJson()
    let( (schCv readCv outputPath defaultPath libName cellName homeDir form)
        
        ; 1. Get current schematic cellview
        schCv = geGetEditCellView()
        
        if( schCv then
            libName = schCv~>libName
            cellName = schCv~>cellName
            
            ; 2. Build default output path
            homeDir = getShellEnvVar("HOME")
            if( !homeDir then homeDir = "/tmp" )
            defaultPath = sprintf(nil "%s/%s.json" homeDir cellName)
            
            ; 3. Create dialog form
            form = hiCreateAppForm(
                ?name 'exportJsonForm
                ?formTitle "Export Schematic to JSON"
                ?fields list(
                    hiCreateStringField(
                        ?name 'libName
                        ?prompt "Library:"
                        ?value libName
                        ?editable nil
                    )
                    hiCreateStringField(
                        ?name 'cellName
                        ?prompt "Cell:"
                        ?value cellName
                        ?editable nil
                    )
                    hiCreateStringField(
                        ?name 'filePath
                        ?prompt "Output JSON file path:"
                        ?value defaultPath
                        ?editable t
                    )
                )
                ?buttonLayout 'OKCancel
            )
            
            ; 4. Display the form and check if user clicked OK
            if( form then
                unless(hiDisplayForm(form)
                    return(nil)  ; User clicked Cancel
                )
                ; 5. Get the output path (remove quotes added by hiCreateStringField)
                outputPath = form->filePath->value
                
                if( outputPath then
                    ; 6. Open a read-only cellview for export (schoa2json will close it)
                    readCv = dbOpenCellViewByType(libName cellName "schematic" "schematic" "r")
                    if( readCv then
                        if( schoa2json(readCv outputPath) then
                            ; Export successful, show success dialog
                            _showSuccessDialog(sprintf(nil "Schematic has been successfully exported to:\n%s" outputPath))
                        else
                            _showErrorDialog("Failed to export schematic to JSON.")
                        )
                    else
                        _showErrorDialog(sprintf(nil "Failed to open schematic view: %s/%s" libName cellName))
                    )
                else
                    _showErrorDialog("Please enter a valid output file path.")
                )
            else
                _showErrorDialog("Failed to create export dialog.")
            )
        else
            _showErrorDialog("No active schematic window found.")
        )
    )
)

/**********************************************
* FUNCTION: schJsonViewMenu
* DESCRIPTION: 
* Creates the pulldown menu object if it doesn't already exist.
* RETURNS: 
* The menu ID symbol.
**********************************************/
procedure( schJsonViewMenu()
    if( boundp( 'ddSchematicJsonExportPulldownMenu ) then
        ddSchematicJsonExportPulldownMenu
    else
        hiCreatePulldownMenu(
            'ddSchematicJsonExportPulldownMenu
            "DrawCKT"  
            list(
                hiCreateMenuItem( 
                    ?name 'ExportSchematicJsonMenu
                    ?itemText "Export Schematic to JSON"
                    ?callback "exportSchematicToJson()"
                )
            ) 
        ) 
    ) 
) 

/**********************************************
* FUNCTION: regSchJsonTriggers
* DESCRIPTION: 
* Callback for deRegUserTriggers. Inserts the custom menu into the banner.
* ARGS:
* args - List of arguments passed by the trigger (includes window ID).
**********************************************/
procedure( _regSchJsonTriggers(args)
    hiInsertBannerMenu(
        args->window
        schJsonViewMenu()
        length( hiGetBannerMenus(args->window) )
    )
)

/**********************************************
* FUNCTION: regSchJsonTriggers
* DESCRIPTION: 
* Main registration function. Registers the trigger for Schematic L and XL.
**********************************************/
procedure( regSchJsonTriggers()
    deRegUserTriggers("schematic"   nil nil '_regSchJsonTriggers)
    deRegUserTriggers("schematicXL" nil nil '_regSchJsonTriggers)
    printf("Schematic to JSON Export Menu Registered.\n")
)

;;; Execution: Register the triggers immediately
regSchJsonTriggers()


;; cv = dbOpenCellViewByType(designLibName cellName "schematic" "schematic" "r")

;==========================================================
; Usage Example
;==========================================================
; Make sure to specify a valid path (e.g., /tmp/ or your home dir)
;; schoa2json("amacro3_0_gen_TB_mos3" "V2I" "./V2I_schematic.json")
