use std::{
    collections::{HashMap, HashSet},
    iter,
};

use macroquad::{
    color::{Color, BLACK, DARKPURPLE, GRAY as GREY, RED, WHITE},
    file::load_file,
    math::{clamp, vec2, Rect},
    shapes::{draw_line, draw_rectangle},
    text::draw_text,
    texture::{draw_texture_ex, DrawTextureParams},
    ui::root_ui,
};
use serde::{Deserialize, Serialize};

use crate::{
    input::Input,
    resources::{
        tilesets::{TileAsset, TileAutoRule, TileLayer, TilesetAsset, TilesetAssetSerializable},
        AssetLoadError,
    },
    utils::{alert, prompt, splitter},
    world::World,
    TILE_COLLISION_SECTIONS, TILE_SIZE, VIRTUAL_H, VIRTUAL_W,
};

use super::tilesets::CollisionMatrix;

pub type TileVec = Vec<Vec<Option<TilePointer>>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TilePointer(String, pub usize);

#[derive(Serialize, Deserialize, Debug)]
struct LevelSerializable {
    background: TileVec,
    object: TileVec,
    overlay: TileVec,
    rows: usize,
    cols: usize,
}
pub struct LevelEditorSettings {
    pub open: bool,
    selected_tileset: Option<String>,
    selected_tile: Option<usize>,
    zoom: Rect,
    pub show_background: bool,
    pub show_object: bool,
    pub show_overlay: bool,
    editing_tile: bool,
}

impl LevelEditorSettings {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_tileset: None,
            selected_tile: None,
            zoom: Rect::new(0.0, 0.0, 0.0, 0.0),
            show_background: true,
            show_object: true,
            show_overlay: true,
            editing_tile: false,
        }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.selected_tile = None;
        self.selected_tileset = None;
    }
}

pub struct TileHitInfo {
    row: f32,
    col: f32,
}

impl TileHitInfo {
    const SMALL: f32 = 0.0001;

    pub fn from_left(&self) -> f32 {
        self.col * TILE_SIZE - Self::SMALL
    }

    pub fn from_right(&self) -> f32 {
        self.col * TILE_SIZE + (TILE_SIZE / TILE_COLLISION_SECTIONS)
    }

    pub fn from_top(&self) -> f32 {
        self.row * TILE_SIZE - Self::SMALL
    }

    pub fn from_bottom(&self) -> f32 {
        self.row * TILE_SIZE + (TILE_SIZE / TILE_COLLISION_SECTIONS)
    }
}

macro_rules! get_tile_mut {
    ($level:expr, $layer_id:expr, $row:expr, $col:expr) => {
        match $layer_id {
            TileLayer::Background => &mut $level.background,
            TileLayer::Object => &mut $level.object,
            TileLayer::Overlay => &mut $level.overlay,
        }
        .get_mut($row)
        .expect("Row should exist")
        .get_mut($col)
        .expect("Tile should exist")
    };
}

pub struct Level {
    rows: usize,
    cols: usize,
    path: String,
    background: TileVec,
    object: TileVec,
    overlay: TileVec,
    tilesets: HashMap<String, TilesetAsset>,
}

impl Level {
    pub async fn load<'a>(level: &str) -> Result<Level, AssetLoadError> {
        let path = format!("assets/levels/{}.json", level);
        let serializable: LevelSerializable = serde_json::from_slice(&load_file(&path).await?)?;
        let mut new = Level {
            background: serializable.background,
            object: serializable.object,
            overlay: serializable.overlay,
            tilesets: HashMap::new(),
            rows: serializable.rows,
            cols: serializable.cols,
            path,
        };

        let mut textures = HashSet::new();
        for row in (&new.background)
            .into_iter()
            .chain(&new.object)
            .chain(&new.overlay)
        {
            for ptr in row {
                if let Some(ptr) = ptr {
                    textures.insert(ptr.0.clone());
                }
            }
        }

        for tex in textures {
            let tiles = TilesetAsset::load(&tex).await?;
            new.tilesets.insert(tex, tiles);
        }

        return Ok(new);
    }

    fn render_layer(&self, layer: &TileVec, world: &World, is_background: bool) {
        let num_rows = (world.h / TILE_SIZE).ceil() as i32;
        let num_cols = (world.w / TILE_SIZE).ceil() as i32;

        let first_row = (world.y / TILE_SIZE).floor() as i32;
        let first_col = (world.x / TILE_SIZE).floor() as i32;

        let row_range = clamp(first_row, 0, self.rows as i32)
            ..clamp(first_row + num_rows + 1, 0, self.rows as i32);
        let col_range = clamp(first_col, 0, self.cols as i32)
            ..clamp(first_col + num_cols + 1, 0, self.cols as i32);

        for row in row_range {
            for col in col_range.clone() {
                let x = col as f32 * TILE_SIZE - world.x;
                let y = row as f32 * TILE_SIZE - world.y;
                if let Some(tile) = &layer[row as usize][col as usize] {
                    let tileset = &self.tilesets[&tile.0];
                    let tile = &tileset.tiles[tile.1];

                    draw_texture_ex(
                        &tileset.tex,
                        x,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: None,
                            source: Some(Rect::new(tile.x, tile.y, TILE_SIZE, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                } else if is_background {
                    draw_rectangle(
                        x,
                        y,
                        TILE_SIZE,
                        TILE_SIZE,
                        Color::from_rgba(150, 0, 150, 255),
                    );
                }
            }
        }
    }

    pub fn render_background(&self, world: &World) {
        self.render_layer(&self.background, world, true);
    }

    pub fn render_object_layer(&self, world: &World) {
        self.render_layer(&self.object, world, false);
    }

    pub fn render_overlay(&self, world: &World) {
        self.render_layer(&self.overlay, world, false);
    }

    pub fn get_layer(&self, layer: &TileLayer) -> &TileVec {
        match layer {
            TileLayer::Background => &self.background,
            TileLayer::Object => &self.object,
            TileLayer::Overlay => &self.overlay,
        }
    }

    pub fn check_for_collision(&self, x: f32, y: f32) -> Option<TileHitInfo> {
        let row = (y / TILE_SIZE).floor();
        let col = (x / TILE_SIZE).floor();

        let tile_ptr = match self.object.get(row as usize) {
            Some(row) => match row.get(col as usize) {
                Some(tile) => match tile {
                    Some(tile) => tile,
                    None => return None,
                },
                None => return None,
            },
            None => return None,
        };

        let tile = &self.tilesets[&tile_ptr.0].tiles[tile_ptr.1];

        let portion_size = TILE_SIZE / TILE_COLLISION_SECTIONS;
        let portion_row = ((y - (row * TILE_SIZE)) / portion_size).floor();
        let portion_col = ((x - (col * TILE_SIZE)) / portion_size).floor();

        match &tile.collision_matrix {
            Some(collision_matrix) => {
                match collision_matrix.matrix[portion_row as usize][portion_col as usize] {
                    true => {
                        return Some(TileHitInfo {
                            row: row + portion_row * (1.0 / TILE_COLLISION_SECTIONS),
                            col: col + portion_col * (1.0 / TILE_COLLISION_SECTIONS),
                        })
                    }
                    false => None,
                }
            }
            None => return None,
        }
    }
}

// EDITOR IMPL
impl Level {
    async fn tile_select_tex(
        &mut self,
        editor: &mut LevelEditorSettings,
        editor_width: f32,
        editor_y: f32,
        input: &Input,
        dt: f32,
    ) -> Result<(), AssetLoadError> {
        if let Some(tileset_id) = &editor.selected_tileset {
            if root_ui().button(None, "Save Tileset Data") {
                if let Some(tileset_id) = &editor.selected_tileset {
                    let serializable = self.tileset_to_serializable(&tileset_id);
                    let msg = match std::fs::write(
                        &self.tilesets[tileset_id].meta_path,
                        serde_json::to_string_pretty(&serializable)?,
                    ) {
                        Ok(_) => "Meta Saved",
                        Err(err) => &format!("{err}"),
                    };

                    alert(msg).await;
                }
            }

            if root_ui().button(None, "Cut Tiles") {
                self.tilesets
                    .get_mut(tileset_id)
                    .expect("Tileset should exist")
                    .cut()
            }

            let tileset = self.tilesets.get(tileset_id).expect("Tileset should exist");
            let ratio_y2x = tileset.tex.height() / tileset.tex.width();
            let ratio_x2y = tileset.tex.width() / tileset.tex.height();

            let dest_size = match ratio_y2x > 1.0 {
                true => Some(vec2(editor_width * ratio_y2x, editor_width)),
                false => Some(vec2(editor_width, editor_width * ratio_x2y)),
            };

            let scroll = input.scroll * dt * 10.0;
            editor.zoom.w += scroll;
            editor.zoom.h += scroll;

            if editor.zoom.w < 2.0
                || editor.zoom.h < 2.0
                || editor.zoom.h > tileset.tex.height()
                || editor.zoom.w > tileset.tex.width()
            {
                editor.zoom.w -= scroll;
                editor.zoom.h -= scroll;
            }

            draw_texture_ex(
                &tileset.tex,
                0.0,
                editor_y,
                WHITE,
                DrawTextureParams {
                    dest_size,
                    source: Some(editor.zoom.clone()),
                    ..Default::default()
                },
            );

            if input.mouse_x < -1.0 / 3.0 {
                let tiles_per_sec = 10.0;
                editor.zoom.x += input.horizontal * dt * TILE_SIZE * tiles_per_sec;
                editor.zoom.x = clamp(editor.zoom.x, 0.0, tileset.tex.width() - editor.zoom.w);

                editor.zoom.y += input.vertical * dt * TILE_SIZE * tiles_per_sec;
                editor.zoom.y = clamp(editor.zoom.y, 0.0, tileset.tex.height() - editor.zoom.h);

                let rm = if input.mouse_x < -1.0 / 3.0
                    && input.mouse_y > editor_width / VIRTUAL_H * 2.0 - 1.0
                {
                    let x = (1.0 + input.mouse_x) / (2.0 / 3.0);
                    Some((x, input.mouse_y))
                } else {
                    None
                };

                if let Some(rm) = rm {
                    let row = ((editor.zoom.h * rm.1 + editor.zoom.y) / TILE_SIZE).floor();
                    let col = ((editor.zoom.w * rm.0 + editor.zoom.x) / TILE_SIZE).floor();

                    let section = Rect::new(col * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE);

                    let scale = editor_width / editor.zoom.w;
                    let x = (section.x - editor.zoom.x) * scale;
                    let y = (section.y - editor.zoom.y) * scale + editor_y;
                    let mut w = TILE_SIZE * scale;
                    let h = w;

                    if x + w > editor_width {
                        w = editor_width - x;
                    }

                    if let Some(tile) = tileset.get_tile_at_pos(section.x, section.y) {
                        draw_rectangle(x, y, w, h, Color::from_rgba(255, 255, 255, 200));
                        if input.click {
                            editor.selected_tile = Some(tile);
                            editor.editing_tile = true;
                        }
                    }
                }
            }
        }

        return Ok(());
    }

    fn draw_panel(&self, editor_width: f32, editor_y: f32) {
        draw_rectangle(0.0, 0.0, editor_width, VIRTUAL_H, DARKPURPLE);

        // Vertical
        draw_line(editor_width, 0.0, editor_width, VIRTUAL_H, 2.0, WHITE);
        draw_line(editor_width, 0.0, editor_width, VIRTUAL_H, 1.0, BLACK);
        draw_line(
            editor_width + 2.0,
            0.0,
            editor_width + 2.0,
            VIRTUAL_H,
            1.0,
            BLACK,
        );

        // Horizontal
        draw_line(0.0, editor_y, editor_width, editor_y, 3.0, BLACK);
        draw_line(0.0, editor_y, editor_width, editor_y, 1.0, WHITE);
    }

    async fn editor_panel(
        &mut self,
        editor: &mut LevelEditorSettings,
    ) -> Result<(), AssetLoadError> {
        if root_ui().button(None, "Save Level") {
            let serializable = self.to_serializable();
            let msg = match std::fs::write(&self.path, serde_json::to_string_pretty(&serializable)?)
            {
                Ok(_) => "Level Saved",
                Err(err) => &format!("{err}"),
            };

            alert(msg).await;
        }
        splitter();

        root_ui().label(None, &format!("Level Size: {}, {}", self.cols, self.rows));

        if root_ui().button(None, "Resize") {
            if let Some(rows) = prompt("Rows").await {
                if let Some(cols) = prompt("Cols").await {
                    match (rows.trim().parse::<usize>(), cols.trim().parse::<usize>()) {
                        (Ok(rows), Ok(cols)) => {
                            self.rows = rows;
                            self.cols = cols;

                            for row in self.background.iter_mut() {
                                row.resize_with(cols, || None);
                            }

                            self.background.resize_with(rows, || {
                                iter::repeat_with(|| None).take(cols).collect()
                            });

                            for row in self.object.iter_mut() {
                                row.resize_with(cols, || None);
                            }

                            self.object.resize_with(rows, || {
                                iter::repeat_with(|| None).take(cols).collect()
                            });

                            for row in self.overlay.iter_mut() {
                                row.resize_with(cols, || None);
                            }

                            self.overlay.resize_with(rows, || {
                                iter::repeat_with(|| None).take(cols).collect()
                            });
                        }
                        _ => {
                            alert(&format!("Could not resize to ({rows}, {cols})")).await;
                        }
                    }
                }
            }
        }
        splitter();

        if root_ui().button(None, "Add tileset") {
            if let Some(tileset_name) = prompt("Tileset Name").await {
                match TilesetAsset::load(&tileset_name).await {
                    Ok(tileset) => {
                        self.tilesets.insert(tileset_name, tileset);
                    }
                    Err(err) => alert(&format!("{err}")).await,
                }
            }
        }
        splitter();

        root_ui().label(None, "Layers");
        let on_off = |x: bool| if x { "On" } else { "Off" };
        if root_ui().button(
            None,
            format!("Toggle Background {}", on_off(editor.show_background)),
        ) {
            editor.show_background = !editor.show_background
        }

        if root_ui().button(
            None,
            format!("Toggle Object {}", on_off(editor.show_object)),
        ) {
            editor.show_object = !editor.show_object
        }

        if root_ui().button(
            None,
            format!("Toggle Overlay {}", on_off(editor.show_overlay)),
        ) {
            editor.show_overlay = !editor.show_overlay
        }

        splitter();

        root_ui().label(None, "Loaded Tilesets");

        for tileset in &self.tilesets {
            if root_ui().button(None, tileset.0.as_str()) {
                let rect = match tileset.1.tex.width() > tileset.1.tex.height() {
                    true => Rect::new(0.0, 0.0, tileset.1.tex.height(), tileset.1.tex.height()),
                    false => Rect::new(0.0, 0.0, tileset.1.tex.width(), tileset.1.tex.width()),
                };
                editor.selected_tileset = Some(tileset.0.clone());
                editor.zoom = rect;
                editor.selected_tile = None;
            }
        }
        splitter();

        let selected = match &editor.selected_tileset {
            Some(tileset) => match editor.selected_tile {
                Some(some) => &format!("{}:{}", tileset, some),
                None => &format!("{}:None", tileset),
            },
            None => "None",
        };

        root_ui().label(None, &format!("Selected: {selected}"));

        return Ok(());
    }

    fn get_tile(&self, tile_ptr: &TilePointer) -> &TileAsset {
        &self.tilesets[&tile_ptr.0].tiles[tile_ptr.1]
    }

    fn get_auto_tile_for_index(
        &self,
        row: usize,
        col: usize,
        layer: &TileLayer,
        group: Option<u8>,
    ) -> TileAutoRule {
        let layer = self.get_layer(layer);

        let i_row = row as i32;
        let i_col = col as i32;

        let present = [
            (i_row - 1, i_col - 1),
            (i_row - 1, i_col),
            (i_row - 1, i_col + 1),
            (i_row, i_col + 1),
            (i_row + 1, i_col + 1),
            (i_row + 1, i_col),
            (i_row + 1, i_col - 1),
            (i_row, i_col - 1),
        ];

        let present = present.map(|(row, col)| {
            match layer.get(if row >= 0 {
                row as usize
            } else {
                return false;
            }) {
                Some(row) => match row.get(if col >= 0 {
                    col as usize
                } else {
                    return false;
                }) {
                    Some(tile) => match tile {
                        Some(tile) => self.get_tile(tile).group == group,
                        None => false,
                    },
                    None => false,
                },
                None => false,
            }
        });

        return TileAutoRule::from_array(present);
    }

    fn find_best_tile_for_index<'a>(
        &'a self,
        row: usize,
        col: usize,
        tile: &'a TileAsset,
        tileset_id: &String,
    ) -> Option<TilePointer> {
        let auto_rule = self.get_auto_tile_for_index(row, col, &tile.layer, tile.group);

        let mut max = (0, None);

        for (idx, possible) in self.tilesets[tileset_id].tiles.iter().enumerate() {
            if possible.group == tile.group {
                if let Some(ref possible_rule) = possible.auto_rule {
                    if let Some(pts) = possible_rule.cmp(&auto_rule) {
                        if pts >= max.0 {
                            max = (pts, Some(TilePointer(tileset_id.clone(), idx)));
                        }
                    }
                }
            }
        }

        return max.1;
    }

    fn set_surrounding_tiles(&mut self, row: usize, col: usize, layer_id: &TileLayer) {
        let i_row = row as i32;
        let i_col = col as i32;
        let sets = [
            (i_row - 1, i_col - 1),
            (i_row - 1, i_col),
            (i_row - 1, i_col + 1),
            (i_row, i_col + 1),
            (i_row + 1, i_col + 1),
            (i_row + 1, i_col),
            (i_row + 1, i_col - 1),
            (i_row, i_col - 1),
        ];

        for set in sets {
            if set.0 >= 0 && set.0 < self.rows as i32 && set.1 >= 0 && set.1 < self.cols as i32 {
                let row = set.0 as usize;
                let col = set.1 as usize;

                let layer = self.get_layer(layer_id);
                if let Some(tile_ptr) = &layer[row][col] {
                    let tile_ptr = self.find_best_tile_for_index(
                        row,
                        col,
                        self.get_tile(tile_ptr),
                        &tile_ptr.0,
                    );

                    if let Some(_) = tile_ptr {
                        *get_tile_mut!(self, layer_id, row, col) = tile_ptr;
                    }
                }
            }
        }
    }

    fn place_tile(&mut self, row: usize, col: usize, editor: &LevelEditorSettings, auto_tile: bool) {
        if let (Some(tileset_id), Some(tile_id)) = (&editor.selected_tileset, editor.selected_tile)
        {
            if auto_tile {
                if let (Some(tileset_id), Some(tile_id)) =
                    (&editor.selected_tileset, editor.selected_tile)
                {
                    let tile = &self.tilesets[tileset_id].tiles[tile_id];
                    let layer = &tile.layer;
                    let tile_ptr = match self.find_best_tile_for_index(row, col, tile, tileset_id) {
                        Some(tile_ptr) => Some(tile_ptr),
                        None => Some(TilePointer(tileset_id.clone(), tile_id)),
                    };

                    *get_tile_mut!(self, layer, row, col) = tile_ptr;
                    self.set_surrounding_tiles(row, col, &layer.clone());
                }
            } else {
                let layer = &self.tilesets[tileset_id].tiles[tile_id].layer;
                *get_tile_mut!(self, layer, row, col) =
                    Some(TilePointer(tileset_id.clone(), tile_id));
            }
        } else {
            if editor.show_background {
                *self
                    .background
                    .get_mut(row)
                    .expect("Row should exist")
                    .get_mut(col)
                    .expect("Col will exist") = None;
                if auto_tile {
                    self.set_surrounding_tiles(row, col, &TileLayer::Background);
                }
            }

            if editor.show_object {
                *self
                    .object
                    .get_mut(row)
                    .expect("Row should exist")
                    .get_mut(col)
                    .expect("Col will exist") = None;
                if auto_tile {
                    self.set_surrounding_tiles(row, col, &TileLayer::Object);
                }
            }

            if editor.show_overlay {
                *self
                    .overlay
                    .get_mut(row)
                    .expect("Row should exist")
                    .get_mut(col)
                    .expect("Col will exist") = None;
                if auto_tile {
                    self.set_surrounding_tiles(row, col, &TileLayer::Overlay);
                }
            }
        }
    }

    fn tile_placer_selector(
        &mut self,
        editor: &mut LevelEditorSettings,
        editor_width: f32,
        input: &Input,
        world: &World,
    ) {
        if input.mouse_x < -1.0 / 3.0 {
            return;
        }

        let mouse = (
            (input.mouse_x + 1.0) / 2.0 * VIRTUAL_W,
            (input.mouse_y + 1.0) / 2.0 * VIRTUAL_H,
        );

        let col = ((mouse.0 + world.x) / TILE_SIZE).floor();
        let row = ((mouse.1 + world.y) / TILE_SIZE).floor();

        let mut x = col * TILE_SIZE - world.x;
        let y = row * TILE_SIZE - world.y;

        let w = if x < editor_width {
            let diff = editor_width - x;
            x = editor_width;
            TILE_SIZE - diff
        } else {
            TILE_SIZE
        };

        if col < 0.0 || col >= self.cols as f32 || row < 0.0 || row >= self.rows as f32 {
            draw_rectangle(x, y, w, TILE_SIZE, RED);
            return;
        } else {
            draw_rectangle(x, y, w, TILE_SIZE, Color::from_rgba(255, 0, 0, 130));
        };

        if let Some(tileset_id) = &editor.selected_tileset {
            if let Some(tile_id) = editor.selected_tile {
                let tileset = &self.tilesets.get(tileset_id).expect("Tileset will exist");
                let tile = &tileset.tiles[tile_id];

                if !input.mouse_down {
                    draw_texture_ex(
                        &tileset.tex,
                        x,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(w, TILE_SIZE)),
                            source: Some(Rect::new(tile.x + TILE_SIZE - w, tile.y, w, TILE_SIZE)),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        if input.mouse_down {
            self.place_tile(row as usize, col as usize, &editor, !input.enter);
        }
    }

    fn edit_tile_collision_matrix(
        tile: &mut TileAsset,
        editor_width: f32,
        editor_y: f32,
        first_cell_x: f32,
        input: &Input,
    ) {
        if let Some(ref mut collision_matrix) = tile.collision_matrix {
            let tile_x = editor_width / TILE_COLLISION_SECTIONS;
            let tile_y = editor_y + tile_x;
            let space = first_cell_x / collision_matrix.matrix.len() as f32;

            for (row_idx, row) in collision_matrix.matrix.iter_mut().enumerate() {
                for (col_idx, tile) in row.iter_mut().enumerate() {
                    let x = tile_x + col_idx as f32 * space;
                    let y = tile_y + row_idx as f32 * space;

                    let mpos = (
                        (input.mouse_x + 1.0) / 2.0 * VIRTUAL_W,
                        (input.mouse_y + 1.0) / 2.0 * VIRTUAL_H,
                    );

                    let hovering =
                        mpos.0 > x && mpos.0 < x + space && mpos.1 < y + space && mpos.1 > y;

                    let color = match hovering {
                        true => GREY,
                        false => WHITE,
                    };

                    let text = match tile {
                        true => "X",
                        false => "O",
                    };

                    draw_text(text, x + 2.0, y + 9.0, 16.0, color);

                    if input.click && hovering {
                        *tile = !*tile
                    }
                }
            }
        }
    }
    fn edit_tile_rules(tile: &mut TileAsset, editor_y: f32, tile_size: f32, input: &Input) {
        if let Some(ref mut auto_rule) = tile.auto_rule {
            let sets = [
                (0, 0, &mut auto_rule.top_left),
                (1, 0, &mut auto_rule.top),
                (2, 0, &mut auto_rule.top_right),
                (2, 1, &mut auto_rule.right),
                (2, 2, &mut auto_rule.bottom_right),
                (1, 2, &mut auto_rule.bottom),
                (0, 2, &mut auto_rule.bottom_left),
                (0, 1, &mut auto_rule.left),
            ];

            for set in sets {
                let x = set.0 as f32 * tile_size;
                let y = set.1 as f32 * tile_size + editor_y;

                let offset = tile_size / 2.0;
                let tx = x + offset - 4.0;
                let ty = y + offset + 4.0;

                let mpos = (
                    ((input.mouse_x + 1.0) / 2.0) * VIRTUAL_W,
                    ((input.mouse_y + 1.0) / 2.0) * VIRTUAL_H,
                );

                let hovering = mpos.0 >= x
                    && mpos.0 <= x + tile_size
                    && mpos.1 >= y
                    && mpos.1 <= y + tile_size;

                let text = match set.2 {
                    Some(true) => "X",
                    Some(false) => "O",
                    None => "?",
                };

                draw_text(
                    text,
                    tx,
                    ty,
                    16.0,
                    match hovering {
                        true => GREY,
                        false => WHITE,
                    },
                );

                if input.click && hovering {
                    *set.2 = match set.2 {
                        Some(true) => Some(false),
                        Some(false) => None,
                        None => Some(true),
                    }
                }
            }
        } else {
            splitter();
            if root_ui().button(None, "Add rules") {
                tile.auto_rule = Some(TileAutoRule::from_array([
                    true, true, true, true, true, true, true, true,
                ]))
            }
        }
    }

    async fn edit_tile_layer(tile: &mut TileAsset) {
        root_ui().label(
            None,
            &format!(
                "Layer: {}",
                match tile.layer {
                    TileLayer::Background => "Background",
                    TileLayer::Object => "Object",
                    TileLayer::Overlay => "Overlay",
                }
            ),
        );

        if root_ui().button(None, "Set Layer") {
            if let Some(layer) = prompt("Layer [B:background/ X:object/ O:overlay]").await {
                match layer.as_str() {
                    "B" => {
                        tile.layer = TileLayer::Background;
                        tile.collision_matrix = None;
                    }
                    "X" => {
                        tile.layer = TileLayer::Object;
                        if let None = tile.collision_matrix {
                            tile.collision_matrix = Some(CollisionMatrix::new());
                        }
                    }
                    "O" => {
                        tile.layer = TileLayer::Overlay;
                        tile.collision_matrix = None
                    }
                    _ => alert("Invalid layer code.").await,
                }
            }
        }
    }

    async fn edit_tile(
        &mut self,
        input: &Input,
        editor: &mut LevelEditorSettings,
        editor_width: f32,
        editor_y: f32,
    ) {
        if let (Some(tileset_id), Some(tile_id)) = (&editor.selected_tileset, editor.selected_tile)
        {
            root_ui().label(None, &format!("{tileset_id}:{tile_id}"));
            splitter();

            if root_ui().button(None, "Deselect Tile") {
                editor.editing_tile = false;
                editor.selected_tile = None;
            }
            splitter();

            let tileset = self
                .tilesets
                .get_mut(tileset_id)
                .expect("Tileset will exist");

            let tile = tileset.tiles.get_mut(tile_id).expect("Tileset will exist");

            root_ui().label(None, &format!("Group: {:?}", tile.group));
            if root_ui().button(None, "Set Group") {
                if let Some(group) = prompt("Group (u8 [0-255])").await {
                    match group.parse() {
                        Ok(group) => tile.group = Some(group),
                        Err(_) => alert("Invalid group u8 [0-255]").await,
                    }
                } else {
                    tile.group = None;
                }
            }
            splitter();

            Self::edit_tile_layer(tile).await;

            let x = editor_width / 3.0;
            let y = editor_y + editor_width / 3.0;
            let size = editor_width / TILE_COLLISION_SECTIONS;

            draw_texture_ex(
                &tileset.tex,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    source: Some(Rect::new(tile.x, tile.y, TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
            );

            Self::edit_tile_rules(tile, editor_y, size, input);
            Self::edit_tile_collision_matrix(tile, editor_width, editor_y, x, input);
        }
    }

    pub async fn level_editor(
        &mut self,
        editor: &mut LevelEditorSettings,
        input: &Input,
        dt: f32,
        world: &World,
    ) -> Result<(), AssetLoadError> {
        let editor_width = VIRTUAL_W / 3.0;
        let editor_y = VIRTUAL_H - editor_width;

        self.draw_panel(editor_width, editor_y);

        if editor.editing_tile {
            self.edit_tile(input, editor, editor_width, editor_y).await;
        } else {
            self.editor_panel(editor).await?;
            self.tile_select_tex(editor, editor_width, editor_y, input, dt)
                .await?;
        }

        self.tile_placer_selector(editor, editor_width, input, world);

        return Ok(());
    }

    fn to_serializable(&self) -> LevelSerializable {
        LevelSerializable {
            background: self.background.clone(),
            object: self.object.clone(),
            overlay: self.overlay.clone(),
            rows: self.rows,
            cols: self.cols,
        }
    }

    fn tileset_to_serializable(&self, tileset_id: &String) -> TilesetAssetSerializable {
        TilesetAssetSerializable {
            tiles: self.tilesets[tileset_id].tiles.clone(),
        }
    }
}
