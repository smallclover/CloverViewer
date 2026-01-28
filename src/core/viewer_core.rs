use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use egui::{Context, TextureHandle};
use lru::LruCache;
use crate::core::image_loader::{ImageLoader, LoadResult};
use crate::core::navigator::Navigator;
use crate::utils::is_image;

pub struct ViewerCore {
    pub loader: ImageLoader,
    pub nav: Navigator,
    pub texture_cache: LruCache<PathBuf, TextureHandle>,
    pub thumb_cache: LruCache<PathBuf, TextureHandle>,
    pub current_texture: Option<TextureHandle>,
    pub error: Option<String>,
    pub zoom: f32,
    pub failed_thumbs: HashSet<PathBuf>,
}

impl ViewerCore {
    pub fn new() -> Self {
        Self {
            loader: ImageLoader::new(),
            nav: Navigator::new(),
            texture_cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
            thumb_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            current_texture: None,
            error: None,
            zoom: 1.0,
            failed_thumbs: HashSet::new(),
        }
    }

    pub fn open_new_context(&mut self, ctx: Context, path: PathBuf) {
        if path.is_dir() {
            self.nav.f_folder(&path);
        } else {
            self.nav.f_image(&path);
        }
        self.load_current(ctx);
    }

    pub fn process_load_results(&mut self, ctx: &Context) -> bool {
        let mut processed_count = 0;
        let mut should_trigger_preloads = false;
        let mut received_any = false;

        while processed_count < 5 {
            match self.loader.rx.try_recv() {
                Ok(msg) => {
                    received_any = true;
                    match msg.result {
                        LoadResult::Ok(tex) => {
                            if msg.is_thumbnail {
                                self.thumb_cache.put(msg.path.clone(), tex.clone());
                                if Some(&msg.path) == self.nav.current().as_ref() {
                                    if self.current_texture.is_none() {
                                        self.current_texture = Some(tex);
                                    }
                                }
                            } else {
                                self.texture_cache.put(msg.path.clone(), tex.clone());
                                if Some(msg.path) == self.nav.current() {
                                    let tex_size = tex.size_vec2();
                                    let available = ctx.available_rect().size();
                                    let scale_v = available.y / tex_size.y;
                                    let scale_h = (available.x - 120.0) / tex_size.x;
                                    self.zoom = scale_v.min(scale_h).min(1.0);

                                    self.current_texture = Some(tex);
                                    self.loader.is_loading = false;
                                    should_trigger_preloads = true;
                                }
                            }
                        }
                        LoadResult::Err(e) => {
                            self.failed_thumbs.insert(msg.path.clone());
                            if msg.is_priority {
                                self.loader.is_loading = false;
                                self.error = Some(e);
                            }
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
        let to_load = self.nav.get_preview_window();
        for (_, path) in to_load {
            if !self.thumb_cache.contains(&path) {
                self.loader.load_async(ctx.clone(), path, false, Some((160, 120)));
            }
        }
    }

    pub fn load_current(&mut self, ctx: Context) {
        self.error = None;
        if let Some(path) = self.nav.current() {
            self.trigger_preloads(&ctx);
            if let Some(tex) = self.texture_cache.get(&path) {
                self.current_texture = Some(tex.clone());
                self.loader.is_loading = false;
            } else {
                if self.failed_thumbs.contains(&path) {
                    self.error = Some("error".to_string());
                    self.current_texture = None;
                    self.loader.is_loading = false;
                } else {
                    self.current_texture = self.thumb_cache.get(&path).cloned();
                    self.loader.load_async(ctx, path, true, None);
                }
            }
        }
    }

    pub fn prev_image(&mut self, ctx: Context) {
        if self.nav.prev().is_some() {
            self.load_current(ctx);
        }
    }

    pub fn next_image(&mut self, ctx: Context) {
        if self.nav.next().is_some() {
            self.load_current(ctx);
        }
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
}
