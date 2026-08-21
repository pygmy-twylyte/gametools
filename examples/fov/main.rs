use eframe::egui::{self, Color32, ComboBox, Pos2, Rect, Sense, Slider, Stroke, StrokeKind, Vec2};
use gametools::{
    FovMap, Grid, GridSize, Point, RectangleFov, perimeter_raycasting_into,
    recursive_shadowcasting_into,
};

const MAP_WIDTH: usize = 220;
const MAP_HEIGHT: usize = 120;
const INITIAL_RANGE: u32 = 28;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "gametools FOV demo",
        options,
        Box::new(|_| Ok(Box::new(FovDemo::new()))),
    )
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    blocks_vision: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FovAlgorithm {
    PerimeterRaycasting,
    RecursiveShadowcasting,
    RectangleBased,
}

impl FovAlgorithm {
    fn label(self) -> &'static str {
        match self {
            Self::PerimeterRaycasting => "Perimeter raycasting",
            Self::RecursiveShadowcasting => "Recursive shadowcasting",
            Self::RectangleBased => "Rectangle-based",
        }
    }
}

struct FovDemo {
    map: Grid<Tile>,
    visible: FovMap,
    rectangle_fov: RectangleFov,
    algorithm: FovAlgorithm,
    hover_point: Option<Point>,
    visible_active: bool,
    visible_count: usize,
    range: u32,
    unlimited_range: bool,
    seed: u32,
}

impl FovDemo {
    fn new() -> Self {
        let seed = 3;
        let map = generate_map(seed);
        let visible = Grid::new(map.size(), false).expect("map size is valid");
        let rectangle_fov = RectangleFov::new(&map, blocks_vision);

        Self {
            map,
            visible,
            rectangle_fov,
            algorithm: FovAlgorithm::RecursiveShadowcasting,
            hover_point: None,
            visible_active: false,
            visible_count: 0,
            range: INITIAL_RANGE,
            unlimited_range: false,
            seed,
        }
    }

    fn regenerate_map(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.map = generate_map(self.seed);
        self.rectangle_fov = RectangleFov::new(&self.map, blocks_vision);
        self.clear_visibility();
    }

    fn fov_radius(&self) -> Option<u32> {
        (!self.unlimited_range).then_some(self.range)
    }

    fn update_visibility(&mut self, source: Point) {
        let radius = self.fov_radius();
        match self.algorithm {
            FovAlgorithm::PerimeterRaycasting => {
                perimeter_raycasting_into(
                    &self.map,
                    &mut self.visible,
                    source,
                    radius,
                    blocks_vision,
                );
            }
            FovAlgorithm::RecursiveShadowcasting => {
                recursive_shadowcasting_into(
                    &self.map,
                    &mut self.visible,
                    source,
                    radius,
                    blocks_vision,
                );
            }
            FovAlgorithm::RectangleBased => {
                self.rectangle_fov
                    .visible_from_into(&mut self.visible, source, radius);
            }
        }

        self.visible_active = true;
        self.visible_count = self.visible.iter().filter(|(_, value)| **value).count();
    }

    fn clear_visibility(&mut self) {
        for (_, value) in self.visible.iter_mut() {
            *value = false;
        }
        self.visible_active = false;
        self.visible_count = 0;
        self.hover_point = None;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("FOV demo");
        ui.label("Hover the map to move the point of view.");
        ui.add_space(8.0);

        ComboBox::from_label("Algorithm")
            .selected_text(self.algorithm.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.algorithm,
                    FovAlgorithm::PerimeterRaycasting,
                    FovAlgorithm::PerimeterRaycasting.label(),
                );
                ui.selectable_value(
                    &mut self.algorithm,
                    FovAlgorithm::RecursiveShadowcasting,
                    FovAlgorithm::RecursiveShadowcasting.label(),
                );
                ui.selectable_value(
                    &mut self.algorithm,
                    FovAlgorithm::RectangleBased,
                    FovAlgorithm::RectangleBased.label(),
                );
            });

        ui.checkbox(&mut self.unlimited_range, "No range limit");
        ui.add_enabled(
            !self.unlimited_range,
            Slider::new(&mut self.range, 1..=max_range()).text("range"),
        );

        if ui.button("Regenerate map").clicked() {
            self.regenerate_map();
        }

        ui.add_space(12.0);
        ui.label(format!("Map: {MAP_WIDTH} x {MAP_HEIGHT}"));
        ui.label(format!(
            "Opaque cells: {:.1}%",
            100.0 * self.opaque_count() as f32 / (MAP_WIDTH * MAP_HEIGHT) as f32
        ));
        ui.label(format!(
            "Blocking rectangles: {}",
            self.rectangle_fov.blocking_rectangles().len()
        ));

        if let Some(point) = self.hover_point {
            ui.label(format!("POV: ({}, {})", point.col, point.row));
            ui.label(format!("Visible cells: {}", self.visible_count));
            if self.map[point].blocks_vision {
                ui.colored_label(
                    Color32::from_rgb(235, 155, 90),
                    "POV is inside an opaque cell",
                );
            }
        } else {
            ui.label("POV: outside map");
        }

        ui.add_space(12.0);
        ui.label("Colors");
        legend_row(ui, Color32::from_rgb(205, 194, 111), "visible floor");
        legend_row(ui, Color32::from_rgb(160, 83, 58), "visible blocker");
        legend_row(ui, Color32::from_rgb(31, 39, 39), "unseen floor");
        legend_row(ui, Color32::from_rgb(12, 17, 19), "unseen blocker");
        legend_row(ui, Color32::from_rgb(44, 214, 167), "point of view");
    }

    fn opaque_count(&self) -> usize {
        self.map
            .iter()
            .filter(|(_, tile)| tile.blocks_vision)
            .count()
    }

    fn map_ui(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let cell_size = (available.x / MAP_WIDTH as f32)
            .min(available.y / MAP_HEIGHT as f32)
            .clamp(4.0, 10.0)
            .floor();
        let desired_size = Vec2::new(MAP_WIDTH as f32 * cell_size, MAP_HEIGHT as f32 * cell_size);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

        let hover_point = response
            .hover_pos()
            .and_then(|pos| point_from_position(rect, cell_size, pos));

        match hover_point {
            Some(point) => {
                self.hover_point = Some(point);
                self.update_visibility(point);
            }
            None if self.visible_active => self.clear_visibility(),
            None => self.hover_point = None,
        }

        self.paint_map(ui, rect, cell_size);
    }

    fn paint_map(&self, ui: &egui::Ui, rect: Rect, cell_size: f32) {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(7, 10, 11));

        for (point, tile) in self.map.iter() {
            let cell_rect = cell_rect(rect, cell_size, point).shrink(0.35);
            let color = tile_color(point, *tile, self.visible_active, self.visible[point]);
            painter.rect_filled(cell_rect, 0.0, color);
        }

        if let Some(point) = self.hover_point {
            let source_rect = cell_rect(rect, cell_size, point).shrink(0.6);
            painter.rect_filled(source_rect, 1.0, Color32::from_rgb(44, 214, 167));
            painter.rect_stroke(
                source_rect,
                1.0,
                Stroke::new(1.4_f32, Color32::WHITE),
                StrokeKind::Inside,
            );
        }

        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0_f32, Color32::from_rgb(85, 95, 90)),
            StrokeKind::Inside,
        );
    }
}

impl eframe::App for FovDemo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.set_height(ui.available_height());

            ui.vertical(|ui| {
                ui.set_width(220.0);
                self.controls(ui);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("Large generated blocker map");
                self.map_ui(ui);
            });
        });
    }
}

fn generate_map(seed: u32) -> Grid<Tile> {
    let size = GridSize::new(MAP_WIDTH, MAP_HEIGHT).expect("example map size is valid");
    Grid::new_with_fn(size, |point| Tile {
        blocks_vision: generated_blocker(point, seed),
    })
    .expect("example map size is valid")
}

fn generated_blocker(point: Point, seed: u32) -> bool {
    let col = point.col;
    let row = point.row;

    if col == 0 || row == 0 || col == MAP_WIDTH as i32 - 1 || row == MAP_HEIGHT as i32 - 1 {
        return true;
    }

    // Long walls with periodic gaps are useful for comparing ray and shadow artifacts.
    let wall = (col == 18 && (5..=63).contains(&row) && row % 13 > 2)
        || (row == 22 && (8..=102).contains(&col) && col % 17 > 3)
        || (col == 76 && (10..=67).contains(&row) && row % 11 > 1);

    // Hard-edged blocks exercise rectangle grouping.
    let rectangles = [
        (8, 7, 14, 15),
        (29, 5, 43, 11),
        (53, 32, 68, 41),
        (86, 8, 101, 18),
        (11, 47, 27, 59),
        (88, 51, 105, 64),
    ];
    let in_rectangle = rectangles
        .into_iter()
        .any(|(min_col, min_row, max_col, max_row)| {
            (min_col..=max_col).contains(&col) && (min_row..=max_row).contains(&row)
        });

    // Rounded and diagonal features expose differences in angular handling.
    let blobs = [
        (41, 50, 8, 5),
        (63, 15, 7, 9),
        (96, 36, 10, 7),
        (35, 31, 5, 11),
    ];
    let in_blob = blobs
        .into_iter()
        .any(|(center_col, center_row, radius_col, radius_row)| {
            let dc = col - center_col;
            let dr = row - center_row;
            dc * dc * radius_row * radius_row + dr * dr * radius_col * radius_col
                <= radius_col * radius_col * radius_row * radius_row
        });

    let strata = ((col * 3 + row * 5 + seed as i32 * 7).rem_euclid(53) == 0)
        && (8..=64).contains(&row)
        && (22..=106).contains(&col);

    wall || in_rectangle || in_blob || strata || hash_noise(point, seed) < 9
}

fn hash_noise(point: Point, seed: u32) -> u32 {
    let mut value = point.col as u32;
    value = value.wrapping_mul(73_856_093) ^ point.row as u32;
    value = value.wrapping_mul(19_349_663) ^ seed.wrapping_mul(83_492_791);
    value ^= value >> 16;
    value = value.wrapping_mul(2_246_822_519);
    value ^= value >> 13;
    value % 100
}

fn blocks_vision(_point: Point, tile: &Tile) -> bool {
    tile.blocks_vision
}

fn max_range() -> u32 {
    MAP_WIDTH.max(MAP_HEIGHT) as u32
}

fn point_from_position(rect: Rect, cell_size: f32, pos: Pos2) -> Option<Point> {
    if !rect.contains(pos) {
        return None;
    }

    let col = ((pos.x - rect.left()) / cell_size).floor() as i32;
    let row = ((pos.y - rect.top()) / cell_size).floor() as i32;
    let point = Point::new(col, row);
    (col >= 0 && row >= 0 && col < MAP_WIDTH as i32 && row < MAP_HEIGHT as i32).then_some(point)
}

fn cell_rect(origin: Rect, cell_size: f32, point: Point) -> Rect {
    let min = Pos2::new(
        origin.left() + point.col as f32 * cell_size,
        origin.top() + point.row as f32 * cell_size,
    );
    Rect::from_min_size(min, Vec2::splat(cell_size))
}

fn tile_color(point: Point, tile: Tile, visible_active: bool, visible: bool) -> Color32 {
    let variation = ((point.col * 11 + point.row * 17).rem_euclid(13)) as u8;
    match (tile.blocks_vision, visible_active, visible) {
        (_, true, true) if tile.blocks_vision => {
            Color32::from_rgb(150 + variation, 74 + variation, 52)
        }
        (_, true, true) => Color32::from_rgb(197 + variation, 188 + variation, 105),
        (true, _, _) => Color32::from_rgb(12, 17, 19 + variation / 3),
        (false, true, false) => Color32::from_rgb(23, 29, 30 + variation / 2),
        (false, _, _) => Color32::from_rgb(38, 47, 45 + variation / 2),
    }
}

fn legend_row(ui: &mut egui::Ui, color: Color32, label: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
        ui.painter().rect_filled(rect, 1.0, color);
        ui.label(label);
    });
}
