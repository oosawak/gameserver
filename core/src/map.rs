use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x + self.width < other.x
            || other.x + other.width < self.x
            || self.y + self.height < other.y
            || other.y + other.height < self.y)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MapElement {
    Wall(Rect),
    Goal(Rect),
    Bush(Rect),
    Spawn(Rect),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<MapElement>,
}

impl Map {
    pub fn new(name: &str, width: f32, height: f32) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            elements: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: MapElement) {
        self.elements.push(element);
    }

    pub fn add_wall(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.add_element(MapElement::Wall(Rect::new(x, y, width, height)));
    }

    pub fn check_wall_collision(&self, x: f32, y: f32, radius: f32) -> bool {
        for element in &self.elements {
            if let MapElement::Wall(rect) = element {
                if x + radius > rect.x
                    && x - radius < rect.x + rect.width
                    && y + radius > rect.y
                    && y - radius < rect.y + rect.height
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn clamp_to_bounds(&self, x: f32, y: f32, radius: f32) -> (f32, f32) {
        let clamped_x = x.max(radius).min(self.width - radius);
        let clamped_y = y.max(radius).min(self.height - radius);
        (clamped_x, clamped_y)
    }
}
