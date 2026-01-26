use crate::{
    DiagramObject,
    DrawrsError::UnsupportedOrient,
    DrawrsResult,
    diagram::text_format::{Justify, JustifyX},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
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
            min_x = min_x.min(bbox.min_x);
            min_y = min_y.min(bbox.min_y);
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
pub struct GroupTransform<'a> {
    origin_bounding_box: BoundingBox,
    offset_x: f64,
    offset_y: f64,
    orient: Orient,
    inst_name: &'a str,
    cell_name: &'a str,
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

impl<'a> GroupTransform<'a> {
    pub fn new(
        origin_bounding_box: BoundingBox,
        offset_x: f64,
        offset_y: f64,
        orient: Orient,
        inst_name: &'a str,
        cell_name: &'a str,
    ) -> Self {
        Self {
            origin_bounding_box,
            offset_x,
            offset_y,
            orient,
            inst_name,
            cell_name,
        }
    }

    fn update_text(&self, text: Option<&mut String>) {
        if let Some(t) = text {
            *t = t.replace("[@cellName]", &self.inst_name);
            *t = t.replace("cdsName()", &self.cell_name);
        }
    }

    /// Transform points from origin coordinates to group-relative coordinates
    /// Points remain in their original coordinates (no transform applied)
    fn update_points<'b, I: Iterator<Item = &'b mut [f64; 2]>>(
        &self,
        points: I,
    ) -> DrawrsResult<()> {
        // Points keep their original coordinates within the group
        for point in points {
            match self.orient {
                Orient::R0 => {}
                Orient::MY => {
                    point[0] = -point[0];
                }
                Orient::R90 => {
                    *point = [point[1], -point[0]];
                }
                Orient::R180 => return Err(UnsupportedOrient(self.orient)),
                Orient::R270 => {
                    *point = [-point[1], point[0]];
                }
                Orient::MX => return Err(UnsupportedOrient(self.orient)),
                Orient::MYR90 => return Err(UnsupportedOrient(self.orient)),
                Orient::MXR90 => return Err(UnsupportedOrient(self.orient)),
            }
            point[0] += self.offset_x;
            point[1] += self.offset_y;
        }
        Ok(())
    }

    /// Transform bounding boxes from origin coordinates to group-relative coordinates
    /// Bounding boxes remain in their original coordinates (no transform applied)
    fn update_box(&self, bbox: Option<(&mut BoundingBox, &mut FlipRotation)>) -> DrawrsResult<()> {
        // Bounding boxes and flip rotations keep their original values within the group
        if let Some((bbox, flip_rotation)) = bbox {
            match self.orient {
                Orient::R0 => {}
                Orient::R90 => {
                    [bbox.min_x, bbox.min_y] = [
                        bbox.min_y - (bbox.width - bbox.height) / 2.0,
                        -bbox.min_x - bbox.width / 2.0 - bbox.height / 2.0,
                    ];
                    flip_rotation.set_rotation(Some(-90.0));
                }
                Orient::R180 => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::R270 => {
                    [bbox.min_x, bbox.min_y] = [
                        -bbox.min_y - (bbox.width + bbox.height) / 2.0,
                        bbox.min_x + bbox.width / 2.0 - bbox.height / 2.0,
                    ];
                    flip_rotation.set_rotation(Some(90.0));
                }
                Orient::MY => {
                    bbox.min_x = -(bbox.min_x + bbox.width);
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
            bbox.min_x += self.offset_x;
            bbox.min_y += self.offset_y;
        }
        Ok(())
    }

    fn update_justify(&self, justify: Option<&mut Justify>) -> DrawrsResult<()> {
        // Bounding boxes and flip rotations keep their original values within the group
        if let Some(justify) = justify {
            match self.orient {
                Orient::R0 => {}
                Orient::R90 => {
                    // return Err(UnsupportedOrient(self.orient));
                }
                Orient::R180 => {
                    return Err(UnsupportedOrient(self.orient));
                }
                Orient::R270 => {
                    // return Err(UnsupportedOrient(self.orient));
                }
                Orient::MY => {
                    justify.x = match justify.x {
                        JustifyX::Left => JustifyX::Right,
                        JustifyX::Center => JustifyX::Center,
                        JustifyX::Right => JustifyX::Left,
                    };
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
        new_obj.set_tag(Some(self.inst_name.to_owned()));

        if let Some(parent) = new_obj.xml_parent() {
            if parent.starts_with("layer-") {
                self.update_points(new_obj.mut_points())?;
                self.update_box(new_obj.mut_box())?;
                self.update_justify(new_obj.justify_mut())?;
            }
        }
        Ok(new_obj)
    }
}
