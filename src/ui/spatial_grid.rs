use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

use crate::model::node::Node;
use crate::ui::node_widget;

/// Uniform-grid spatial hash for fast node lookups by position.
/// Cells are `cell_size × cell_size` in canvas coordinates.
#[derive(Debug, Clone)]
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<Uuid>>,
    dirty: bool,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new(300.0)
    }
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            dirty: true,
        }
    }

    /// Mark the grid as needing a full rebuild.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Rebuild the grid from scratch if dirty. Returns whether a rebuild occurred.
    pub fn rebuild_if_dirty(&mut self, nodes: &BTreeMap<Uuid, Node>) -> bool {
        if !self.dirty {
            return false;
        }
        self.cells.clear();
        for (&id, node) in nodes {
            let rect = node_widget::node_rect(node);
            let min_cell = self.cell_coords(rect.min.x, rect.min.y);
            let max_cell = self.cell_coords(rect.max.x, rect.max.y);
            for cx in min_cell.0..=max_cell.0 {
                for cy in min_cell.1..=max_cell.1 {
                    self.cells.entry((cx, cy)).or_default().push(id);
                }
            }
        }
        self.dirty = false;
        true
    }

    /// Return candidate node IDs near a canvas-space point.
    /// Checks the point's cell and all 8 neighbors for robustness.
    pub fn query_point(&self, x: f32, y: f32) -> Vec<Uuid> {
        let (cx, cy) = self.cell_coords(x, y);
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(ids) = self.cells.get(&(cx + dx, cy + dy)) {
                    for &id in ids {
                        if seen.insert(id) {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// Return candidate node IDs overlapping a canvas-space rectangle.
    pub fn query_rect(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Vec<Uuid> {
        let min_cell = self.cell_coords(min_x, min_y);
        let max_cell = self.cell_coords(max_x, max_y);
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for &id in ids {
                        if seen.insert(id) {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        ((x / self.cell_size).floor() as i32, (y / self.cell_size).floor() as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn empty_grid_returns_nothing() {
        let grid = SpatialGrid::new(300.0);
        assert!(grid.query_point(100.0, 100.0).is_empty());
        assert!(grid.query_rect(0.0, 0.0, 500.0, 500.0).is_empty());
    }

    #[test]
    fn rebuild_and_query_point() {
        let mut grid = SpatialGrid::new(300.0);
        let mut nodes = BTreeMap::new();
        let n1 = Node::new_dialogue([100.0, 100.0]);
        let n2 = Node::new_dialogue([800.0, 800.0]);
        let id1 = n1.id;
        let id2 = n2.id;
        nodes.insert(n1.id, n1);
        nodes.insert(n2.id, n2);
        grid.rebuild_if_dirty(&nodes);

        let near1 = grid.query_point(150.0, 120.0);
        assert!(near1.contains(&id1));
        assert!(!near1.contains(&id2));

        let near2 = grid.query_point(850.0, 820.0);
        assert!(near2.contains(&id2));
        assert!(!near2.contains(&id1));
    }

    #[test]
    fn query_rect_finds_overlapping() {
        let mut grid = SpatialGrid::new(300.0);
        let mut nodes = BTreeMap::new();
        let n1 = Node::new_dialogue([50.0, 50.0]);
        let n2 = Node::new_dialogue([400.0, 400.0]);
        let n3 = Node::new_dialogue([2000.0, 2000.0]);
        let (id1, id2, id3) = (n1.id, n2.id, n3.id);
        nodes.insert(n1.id, n1);
        nodes.insert(n2.id, n2);
        nodes.insert(n3.id, n3);
        grid.rebuild_if_dirty(&nodes);

        let result = grid.query_rect(0.0, 0.0, 600.0, 600.0);
        assert!(result.contains(&id1));
        assert!(result.contains(&id2));
        assert!(!result.contains(&id3));
    }

    #[test]
    fn dirty_flag() {
        let mut grid = SpatialGrid::new(300.0);
        let nodes = BTreeMap::new();
        assert!(grid.rebuild_if_dirty(&nodes));
        assert!(!grid.rebuild_if_dirty(&nodes));
        grid.mark_dirty();
        assert!(grid.rebuild_if_dirty(&nodes));
    }
}
