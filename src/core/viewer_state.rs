use crate::core::image_loader::{ImageLoadError, ImageLoader, LoadResult};
use crate::model::image_meta::ImageProperties;
use crate::utils::image::{collect_images, is_image};
use egui::{Color32, Context, TextureHandle};
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub enum ViewMode {
    Single,
    Grid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    None,
    WaitNext,
    FadeIn,
}

pub struct ViewerState {
    pub loader: ImageLoader,
    pub list: Vec<PathBuf>,
    pub index: usize,
    pub texture_cache: LruCache<PathBuf, TextureHandle>,
    pub thumb_cache: LruCache<PathBuf, TextureHandle>,
    pub current_texture: Option<TextureHandle>,
    pub current_texture_path: Option<PathBuf>,
    pub current_properties: Option<ImageProperties>,
    pub current_raw_pixels: Option<Arc<Vec<Color32>>>,
    pub error: Option<ImageLoadError>,
    pub zoom: f32,
    pub last_view_size: Option<egui::Vec2>,
    pub failed_thumbs: HashSet<PathBuf>,
    pub loading_thumbs: HashSet<PathBuf>,
    pub previous_texture: Option<TextureHandle>,
    pub previous_zoom: Option<f32>,
    pub transition_phase: TransitionPhase,
    pub transition_phase_start_time: Option<f64>,
    pub transition_target_path: Option<PathBuf>,
    pub view_mode: ViewMode,
}

impl ViewerState {
    pub fn new() -> Self {
        Self {
            loader: ImageLoader::new(),
            list: Vec::new(),
            index: 0,
            texture_cache: LruCache::new(NonZeroUsize::new(10).expect("10 is non-zero")),
            thumb_cache: LruCache::new(NonZeroUsize::new(1000).expect("1000 is non-zero")),
            current_texture: None,
            current_texture_path: None,
            current_properties: None,
            current_raw_pixels: None,
            error: None,
            zoom: 1.0,
            last_view_size: None,
            failed_thumbs: HashSet::new(),
            loading_thumbs: HashSet::new(),
            previous_texture: None,
            previous_zoom: None,
            transition_phase: TransitionPhase::None,
            transition_phase_start_time: None,
            transition_target_path: None,
            view_mode: ViewMode::Single,
        }
    }

    pub fn f_image(&mut self, path: &Path) {
        if let Some(dir) = path.parent() {
            let mut v = collect_images(dir);
            v.sort();
            self.index = v.iter().position(|p| p == path).unwrap_or(0);
            self.list = v;
        }
    }

    pub fn f_folder(&mut self, path: &Path) {
        let mut v = collect_images(path);
        v.sort();
        self.index = 0;
        self.list = v;
    }

    pub fn next(&mut self) -> Option<PathBuf> {
        if self.list.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.list.len();
        self.current()
    }

    pub fn prev(&mut self) -> Option<PathBuf> {
        if self.list.is_empty() {
            return None;
        }
        self.index = if self.index == 0 {
            self.list.len() - 1
        } else {
            self.index - 1
        };
        self.current()
    }

    pub fn current(&self) -> Option<PathBuf> {
        self.list.get(self.index).cloned()
    }

    pub fn get_preview_window(&self) -> Vec<(usize, PathBuf)> {
        if self.list.is_empty() {
            return Vec::new();
        }

        let len = self.list.len();
        let mut result = Vec::new();

        for offset in -2..=2 {
            let idx = (self.index as isize + offset).rem_euclid(len as isize) as usize;
            if let Some(path) = self.list.get(idx) {
                result.push((idx, path.clone()));
            }
        }

        result
    }

    pub fn set_index(&mut self, index: usize) -> Option<PathBuf> {
        if index < self.list.len() {
            self.index = index;
            self.current()
        } else {
            None
        }
    }

    pub fn open_new_context(&mut self, ctx: Context, path: PathBuf) {
        self.current_texture = None;
        self.current_texture_path = None;
        self.current_properties = None;
        self.current_raw_pixels = None;
        self.previous_texture = None;
        self.previous_zoom = None;
        self.transition_phase = TransitionPhase::None;
        self.transition_phase_start_time = None;
        self.transition_target_path = None;

        if path.is_dir() {
            self.f_folder(&path);
            self.view_mode = ViewMode::Grid;
        } else {
            self.f_image(&path);
            self.view_mode = ViewMode::Single;
        }
        self.load_current(ctx);
    }

    pub fn process_load_results(&mut self, ctx: &Context) -> bool {
        let mut processed_count = 0;
        let mut should_trigger_preloads = false;
        let mut received_any = false;

        while processed_count <= 50 {
            match self.loader.rx.try_recv() {
                Ok(msg) => {
                    received_any = true;
                    match msg.result {
                        LoadResult::Ok(success) => {
                            let success = *success;
                            if msg.is_thumbnail {
                                self.loading_thumbs.remove(&msg.path);
                                self.thumb_cache
                                    .put(msg.path.clone(), success.texture.clone());
                                if Some(msg.path.clone()) == self.current()
                                    && !self.texture_cache.contains(&msg.path)
                                {
                                    self.current_texture = Some(success.texture);
                                    self.current_texture_path = Some(msg.path);
                                }
                            } else {
                                self.current_raw_pixels = Some(success.raw_pixels);
                                self.texture_cache
                                    .put(msg.path.clone(), success.texture.clone());
                                if Some(msg.path) == self.current() {
                                    let new_zoom =
                                        self.calc_fit_zoom(ctx, success.texture.size_vec2());
                                    self.zoom = new_zoom;

                                    self.current_texture = Some(success.texture);
                                    self.current_texture_path = self.current();
                                    self.current_properties = Some(success.properties);
                                    self.loader.is_loading = false;
                                    should_trigger_preloads = true;
                                }
                            }
                        }
                        LoadResult::Err(ref e) => {
                            self.loading_thumbs.remove(&msg.path);
                            self.failed_thumbs.insert(msg.path.clone());
                            if msg.is_priority {
                                self.loader.is_loading = false;
                                self.error = Some(e.clone());
                            }
                            if Some(msg.path.clone()) == self.current() {
                                self.current_texture = None;
                                self.current_texture_path = None;
                            }
                            tracing::warn!("图片加载失败 {}: {}", msg.path.display(), e);
                        }
                    }
                    processed_count += 1;
                }
                Err(_) => break,
            }
        }

        if should_trigger_preloads {
            self.trigger_preloads(ctx);
        }

        received_any
    }

    pub fn trigger_preloads(&mut self, ctx: &Context) {
        let to_load = self.get_preview_window();
        for (_, path) in to_load {
            if self.thumb_cache.contains(&path)
                || self.failed_thumbs.contains(&path)
                || self.loading_thumbs.contains(&path)
            {
                continue;
            }
            self.loading_thumbs.insert(path.clone());
            self.loader
                .load_async(ctx.clone(), path, false, Some((160, 120)));
        }
    }

    pub fn load_current(&mut self, ctx: Context) {
        self.error = None;
        if let Some(path) = self.current() {
            self.trigger_preloads(&ctx);
            let cached_tex = self.texture_cache.get(&path).cloned();
            if let Some(tex) = cached_tex {
                let new_zoom = self.calc_fit_zoom(&ctx, tex.size_vec2());
                self.zoom = new_zoom;
                self.current_texture = Some(tex.clone());
                self.current_texture_path = Some(path);
                self.loader.is_loading = false;
            } else {
                if self.failed_thumbs.contains(&path) {
                    self.error = Some(ImageLoadError::DecodeError("缩略图加载失败".to_string()));
                    self.current_texture = None;
                    self.current_texture_path = None;
                    self.loader.is_loading = false;
                } else {
                    if let Some(thumb) = self.thumb_cache.get(&path).cloned() {
                        self.current_texture = Some(thumb);
                        self.current_texture_path = Some(path.clone());
                    }
                    self.loader.load_async(ctx, path, true, None);
                }
            }
        } else {
            self.current_texture = None;
            self.current_texture_path = None;
        }
    }

    pub fn prev_image(&mut self, ctx: Context) {
        if self.prev().is_some() {
            self.start_transition(&ctx);
            self.load_current(ctx);
        }
    }

    pub fn next_image(&mut self, ctx: Context) {
        if self.next().is_some() {
            self.start_transition(&ctx);
            self.load_current(ctx);
        }
    }

    pub fn jump_to_index(&mut self, ctx: Context, index: usize) {
        if index != self.index && index < self.list.len() {
            self.set_index(index);
            self.start_transition(&ctx);
            self.load_current(ctx);
        }
    }

    fn start_transition(&mut self, ctx: &Context) {
        self.previous_texture = None;
        self.previous_zoom = None;
        self.current_texture = None;
        self.current_texture_path = None;
        self.transition_phase = TransitionPhase::WaitNext;
        self.transition_phase_start_time = Some(ctx.input(|i| i.time));
        self.transition_target_path = self.current();
    }

    pub fn handle_dropped_file(&mut self, ctx: Context, path: PathBuf) {
        if is_image(&path) {
            self.open_new_context(ctx, path);
        }
    }

    pub fn update_zoom(&mut self, delta: f32) {
        if delta != 0.0 {
            self.zoom = (self.zoom + delta * 0.001).clamp(0.1, 10.0);
        }
    }

    pub(crate) fn calc_fit_zoom(&self, ctx: &Context, tex_size: egui::Vec2) -> f32 {
        let available = self.last_view_size.unwrap_or_else(|| ctx.content_rect().size());
        let scale_v = available.y / tex_size.y;
        let scale_h = (available.x - 120.0) / tex_size.x;

        scale_v.min(scale_h).min(1.0) * 0.9
    }
}
