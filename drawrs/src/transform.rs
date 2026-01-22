use std::fmt;

use crate::{DiagramObject, DrawrsError::UnsupportedOrient, DrawrsResult};

#[derive(Debug, Clone, Copy)]
pub enum Orient {
    R0,
    R90,
    R180,
    R270,
    MY,
    MX,
    MYR90,
    MXR90,
}

impl fmt::Display for Orient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Orient::R0 => "R0",
                Orient::R90 => "R90",
                Orient::R180 => "R180",
                Orient::R270 => "R270",
                Orient::MY => "MY",
                Orient::MX => "MX",
                Orient::MYR90 => "MYR90",
                Orient::MXR90 => "MXR90",
            }
        )
    }
}

impl Orient {
    pub fn from_str(s: &str) -> Self {
        match s {
            "R0" => Orient::R0,
            "R90" => Orient::R90,
            "R180" => Orient::R180,
            "R270" => Orient::R270,
            "MY" => Orient::MY,
            "MX" => Orient::MX,
            "MYR90" => Orient::MYR90,
            "MXR90" => Orient::MXR90,
            _ => Orient::R0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
}

impl BoundingBox {
    pub fn new(min_x: f64, min_y: f64, width: f64, height: f64) -> Self {
        Self {
            min_x,
            min_y,
            width,
            height,
        }
    }

    pub fn min_x(&self) -> f64 {
        self.min_x
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn max_x(&self) -> f64 {
        self.min_x + self.width
    }

    pub fn max_y(&self) -> f64 {
        self.min_y + self.height
    }
    pub fn union(bboxs: impl Iterator<Item = Self>) -> Option<Self> {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut has_objects = false;

        for bbox in bboxs {
            has_objects = true;
            min_x = min_x.min(bbox.min_x());
            min_y = min_y.min(bbox.min_y());
            max_x = max_x.max(bbox.max_x());
            max_y = max_y.max(bbox.max_y());
        }

        if has_objects {
            Some(BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }
}
pub struct GroupTransform {
    origin_bounding_box: BoundingBox,
    offset_x: f64,
    offset_y: f64,
    orient: Orient,
    inst_name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct FlipRotation {
    flip_h: Option<usize>,
    flip_v: Option<usize>,
    legacy_anchor_points: Option<usize>,
    rotation: Option<f64>,
}

impl Default for FlipRotation {
    fn default() -> Self {
        Self {
            flip_h: None,
            flip_v: None,
            legacy_anchor_points: None,
            rotation: None,
        }
    }
}

impl FlipRotation {
    pub fn flip_h(&self) -> Option<usize> {
        self.flip_h
    }

    pub fn flip_v(&self) -> Option<usize> {
        self.flip_v
    }

    pub fn legacy_anchor_points(&self) -> Option<usize> {
        self.legacy_anchor_points
    }

    pub fn rotation(&self) -> Option<f64> {
        self.rotation
    }

    pub fn set_flip_h(&mut self, flip_h: Option<usize>) {
        self.flip_h = flip_h;
    }

    pub fn set_flip_v(&mut self, flip_v: Option<usize>) {
        self.flip_v = flip_v;
    }

    pub fn set_legacy_anchor_points(&mut self, legacy_anchor_points: Option<usize>) {
        self.legacy_anchor_points = legacy_anchor_points;
    }

    pub fn set_rotation(&mut self, rotation: Option<f64>) {
        self.rotation = rotation;
    }
}

impl GroupTransform {
    pub fn new(
        origin_bounding_box: BoundingBox,
        offset_x: f64,
        offset_y: f64,
        orient: Orient,
        inst_name: String,
    ) -> Self {
        Self {
            origin_bounding_box,
            offset_x,
            offset_y,
            orient,
            inst_name,
        }
    }

    fn update_text(&self, text: Option<&mut String>) {
        if let Some(t) = text {
            *t = t.replace("[@cellName]", &self.inst_name);
        }
    }

    /// Transform points from origin coordinates to group-relative coordinates
    /// Points remain in their original coordinates (no transform applied)
    fn update_points<'a, I: Iterator<Item = &'a mut [f64; 2]>>(
        &self,
        points: I,
    ) -> DrawrsResult<()> {
        // Points keep their original coordinates within the group
        for point in points {
            match self.orient {
                Orient::R0 => {
                    point[0] += self.offset_x;
                    point[1] += self.offset_y;
                }
                Orient::MY => {
                    point[0] = -point[0];
                    point[0] += self.offset_x;
                    point[1] += self.offset_y;
                }
                _ => {
                    return Err(UnsupportedOrient(self.orient));
                }
            }
        }
        Ok(())
    }

    /// Transform bounding boxes from origin coordinates to group-relative coordinates
    /// Bounding boxes remain in their original coordinates (no transform applied)
    fn update_box(&self, bbox: Option<(&mut BoundingBox, &mut FlipRotation)>) -> DrawrsResult<()> {
        // Bounding boxes and flip rotations keep their original values within the group
        if let Some((bbox, flip_rotation)) = bbox {
            match self.orient {
                Orient::R0 => {
                    bbox.min_x += self.offset_x;
                    bbox.min_y += self.offset_y;
                }
                Orient::R90 => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::R180 => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::R270 => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::MY => {
                    bbox.min_x = -(bbox.min_x + bbox.width());
                    bbox.min_x += self.offset_x;
                    bbox.min_y += self.offset_y;
                }
                Orient::MX => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::MYR90 => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::MXR90 => {
                    return Err(UnsupportedOrient(self.orient));
                }
            }
        }
        Ok(())
    }
    pub fn new_obj(&self, obj: &DiagramObject) -> DrawrsResult<DiagramObject> {
        let mut new_obj: DiagramObject = obj.clone();
        new_obj.set_id(format!("{}-{}", self.inst_name, new_obj.id()));
        self.update_text(new_obj.text_mut());
        new_obj.set_tag(Some(self.inst_name.clone()));
        if let Some(parent) = new_obj.xml_parent() {
            if parent.starts_with("layer-") {
                self.update_points(new_obj.mut_points())?;
                self.update_box(new_obj.mut_box())?;
            }
        }
        Ok(new_obj)
    }
}
