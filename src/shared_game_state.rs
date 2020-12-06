use std::ops::Div;
use std::time::Instant;

use bitvec::vec::BitVec;
use chrono::{Datelike, Local};
use ggez::{Context, filesystem, GameResult, graphics};
use ggez::filesystem::OpenOptions;
use ggez::graphics::Canvas;
use gilrs::Gilrs;
use num_traits::clamp;

use crate::bmfont_renderer::BMFontRenderer;
use crate::caret::{Caret, CaretType};
use crate::common::{ControlFlags, Direction, FadeState};
use crate::engine_constants::EngineConstants;
use crate::input::touch_controls::TouchControls;
use crate::npc::{NPC, NPCTable};
use crate::profile::GameProfile;
use crate::rng::RNG;
use crate::scene::game_scene::GameScene;
use crate::scene::Scene;
use crate::settings::{Settings, ControllerType};
use crate::shaders::Shaders;
use crate::sound::SoundManager;
use crate::stage::StageData;
use crate::str;
use crate::text_script::{ScriptMode, TextScriptExecutionState, TextScriptVM};
use crate::texture_set::{g_mag, TextureSet};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TimingMode {
    _50Hz,
    _60Hz,
    FrameSynchronized,
}

impl TimingMode {
    pub fn get_delta(self) -> usize {
        match self {
            TimingMode::_50Hz => { 1000000000 / 50 }
            TimingMode::_60Hz => { 1000000000 / 60 }
            TimingMode::FrameSynchronized => { 0 }
        }
    }

    pub fn get_delta_millis(self) -> f64 {
        match self {
            TimingMode::_50Hz => { 1000.0 / 50.0 }
            TimingMode::_60Hz => { 1000.0 / 60.0 }
            TimingMode::FrameSynchronized => { 0.0 }
        }
    }

    pub fn get_tps(self) -> usize {
        match self {
            TimingMode::_50Hz => { 50 }
            TimingMode::_60Hz => { 60 }
            TimingMode::FrameSynchronized => { 0 }
        }
    }
}


#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Season {
    None,
    Halloween,
    Christmas,
}

impl Season {
    pub fn current() -> Season {
        let now = Local::now();

        if (now.month() == 10 && now.day() > 25) || (now.month() == 11 && now.day() < 3) {
            Season::Halloween
        } else if (now.month() == 12 && now.day() > 23) || (now.month() == 0 && now.day() < 7) {
            Season::Christmas
        } else {
            Season::None
        }
    }
}

pub struct SharedGameState {
    pub timing_mode: TimingMode,
    pub control_flags: ControlFlags,
    pub game_flags: BitVec,
    pub fade_state: FadeState,
    /// RNG used by game state, using it for anything else might cause unintended side effects and break replays.
    pub game_rng: RNG,
    /// RNG used by graphics effects that aren't dependent on game's state.
    pub effect_rng: RNG,
    pub quake_counter: u16,
    pub teleporter_slots: Vec<(u16, u16)>,
    pub carets: Vec<Caret>,
    pub touch_controls: TouchControls,
    pub base_path: String,
    pub npc_table: NPCTable,
    pub npc_super_pos: (isize, isize),
    pub stages: Vec<StageData>,
    pub new_npcs: Vec<NPC>,
    pub frame_time: f64,
    pub scale: f32,
    pub shaders: Shaders,
    pub tmp_canvas: Canvas,
    pub game_canvas: Canvas,
    pub lightmap_canvas: Canvas,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
    pub gilrs: Option<Gilrs>,
    pub textscript_vm: TextScriptVM,
    pub season: Season,
    pub constants: EngineConstants,
    pub font: BMFontRenderer,
    pub texture_set: TextureSet,
    pub sound_manager: SoundManager,
    pub settings: Settings,
    pub shutdown: bool,
}

impl SharedGameState {
    pub fn new(ctx: &mut Context) -> GameResult<SharedGameState> {
        let screen_size = graphics::drawable_size(ctx);
        let scale = screen_size.1.div(235.0).floor().max(1.0);
        unsafe { g_mag = scale };

        let canvas_size = (screen_size.0 / scale, screen_size.1 / scale);

        let mut constants = EngineConstants::defaults();
        let mut base_path = "/";
        let mut settings = Settings::load(ctx)?;
        let mut gilrs = Gilrs::new().ok();

        if let Some(gilrs) = gilrs.as_mut() {
            for (id, pad) in gilrs.gamepads() {
                log::info!("Found gamepad: {} ({})", pad.name(), id);
            }

            if let Some((id, _)) = gilrs.gamepads().next() {
                settings.player2_controller_type = ControllerType::Gamepad(id.into());
            }
        }

        if filesystem::exists(ctx, "/base/Nicalis.bmp") {
            info!("Cave Story+ (PC) data files detected.");
            constants.apply_csplus_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/base/lighting.tbl") {
            info!("Cave Story+ (Switch) data files detected.");
            constants.apply_csplus_patches();
            constants.apply_csplus_nx_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/mrmap.bin") {
            info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") {
            info!("NXEngine-evo data files detected.");
        }

        let font = BMFontRenderer::load(base_path, &constants.font_path, ctx)
            .or_else(|_| BMFontRenderer::load("/", "builtin/builtin_font.fnt", ctx))?;
        let season = Season::current();
        let mut texture_set = TextureSet::new(base_path);

        if constants.is_cs_plus {
            texture_set.apply_seasonal_content(season, &settings);
        }

        println!("lookup path: {:#?}", texture_set.paths);

        Ok(SharedGameState {
            timing_mode: TimingMode::_50Hz,
            control_flags: ControlFlags(0),
            game_flags: bitvec::bitvec![0; 8000],
            fade_state: FadeState::Hidden,
            game_rng: RNG::new(0),
            effect_rng: RNG::new(Instant::now().elapsed().as_nanos() as i32),
            quake_counter: 0,
            teleporter_slots: Vec::with_capacity(8),
            carets: Vec::with_capacity(32),
            touch_controls: TouchControls::new(),
            base_path: str!(base_path),
            npc_table: NPCTable::new(),
            npc_super_pos: (0, 0),
            stages: Vec::with_capacity(96),
            new_npcs: Vec::with_capacity(8),
            frame_time: 0.0,
            scale,
            shaders: Shaders::new(ctx)?,
            tmp_canvas: Canvas::with_window_size(ctx)?,
            game_canvas: Canvas::with_window_size(ctx)?,
            lightmap_canvas: Canvas::with_window_size(ctx)?,
            screen_size,
            canvas_size,
            next_scene: None,
            gilrs,
            textscript_vm: TextScriptVM::new(),
            season,
            constants,
            font,
            texture_set,
            sound_manager: SoundManager::new(ctx)?,
            settings,
            shutdown: false,
        })
    }

    pub fn reload_textures(&mut self) {
        let mut texture_set = TextureSet::new(self.base_path.as_str());

        if self.constants.is_cs_plus {
            texture_set.apply_seasonal_content(self.season, &self.settings);
        }

        self.texture_set = texture_set;
    }

    pub fn start_new_game(&mut self, ctx: &mut Context) -> GameResult {
        let mut next_scene = GameScene::new(self, ctx, 13)?;
        next_scene.player1.cond.set_alive(true);
        next_scene.player1.x = 10 * 16 * 0x200;
        next_scene.player1.y = 8 * 16 * 0x200;
        self.fade_state = FadeState::Hidden;
        self.textscript_vm.state = TextScriptExecutionState::Running(200, 0);

        self.next_scene = Some(Box::new(next_scene));

        Ok(())
    }

    pub fn start_intro(&mut self, ctx: &mut Context) -> GameResult {
        let mut next_scene = GameScene::new(self, ctx, 72)?;
        next_scene.player1.cond.set_hidden(true);
        next_scene.player1.x = 3 * 16 * 0x200;
        next_scene.player1.y = 3 * 16 * 0x200;
        next_scene.intro_mode = true;
        self.fade_state = FadeState::Hidden;
        self.textscript_vm.state = TextScriptExecutionState::Running(100, 0);

        self.next_scene = Some(Box::new(next_scene));

        Ok(())
    }

    pub fn save_game(&mut self, game_scene: &mut GameScene, ctx: &mut Context) -> GameResult {
        if let Ok(data) = filesystem::open_options(ctx, "/Profile.dat", OpenOptions::new().write(true).create(true)) {
            let profile = GameProfile::dump(self, game_scene);
            profile.write_save(data)?;
        } else {
            log::warn!("Cannot open save file.");
        }

        Ok(())
    }

    pub fn load_or_start_game(&mut self, ctx: &mut Context) -> GameResult {
        if let Ok(data) = filesystem::user_open(ctx, "/Profile.dat") {
            match GameProfile::load_from_save(data) {
                Ok(profile) => {
                    self.reset();
                    let mut next_scene = GameScene::new(self, ctx, profile.current_map as usize)?;

                    profile.apply(self, &mut next_scene, ctx);

                    self.next_scene = Some(Box::new(next_scene));
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Failed to load save game, starting new one: {}", e);
                }
            }
        } else {
            log::warn!("No save game found, starting new one...");
        }

        self.start_new_game(ctx)
    }

    pub fn reset(&mut self) {
        self.control_flags.0 = 0;
        self.game_flags = bitvec::bitvec![0; 8000];
        self.fade_state = FadeState::Hidden;
        self.game_rng = RNG::new(0);
        self.teleporter_slots.clear();
        self.quake_counter = 0;
        self.carets.clear();
        self.new_npcs.clear();
        self.textscript_vm.set_mode(ScriptMode::Map);
        self.textscript_vm.suspend = true;
    }

    pub fn handle_resize(&mut self, ctx: &mut Context) -> GameResult {
        self.screen_size = graphics::drawable_size(ctx);
        self.scale = self.screen_size.1.div(240.0).floor().max(1.0);
        self.canvas_size = (self.screen_size.0 / self.scale, self.screen_size.1 / self.scale);
        unsafe { g_mag = self.scale };

        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, self.screen_size.0, self.screen_size.1))?;

        Ok(())
    }

    pub fn tick_carets(&mut self) {
        for caret in self.carets.iter_mut() {
            caret.tick(&self.effect_rng, &self.constants);
        }

        self.carets.retain(|c| !c.is_dead());
    }

    pub fn create_caret(&mut self, x: isize, y: isize, ctype: CaretType, direct: Direction) {
        self.carets.push(Caret::new(x, y, ctype, direct, &self.constants));
    }

    pub fn set_speed(&mut self, value: f64) {
        self.settings.speed = clamp(value, 0.1, 3.0);
        self.frame_time = 0.0;

        if let Err(err) = self.sound_manager.set_speed(value as f32) {
            log::error!("Error while sending a message to sound manager: {}", err);
        }
    }

    pub fn current_tps(&self) -> f64 {
        self.timing_mode.get_tps() as f64 * self.settings.speed
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
    }
}
