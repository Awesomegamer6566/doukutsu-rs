use std::borrow::{Borrow, BorrowMut};

use num_traits::abs;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::inventory::{AddExperienceResult, Inventory};
use crate::npc::{NPC, NPCMap};
use crate::physics::PhysicalEntity;
use crate::player::{ControlMode, Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;

impl PhysicalEntity for Player {
    #[inline(always)]
    fn x(&self) -> isize {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> isize {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> isize {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> isize {
        self.vel_y
    }

    fn hit_rect_size(&self) -> usize {
        2
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &Rect<usize> {
        &self.hit_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: isize) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: isize) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: isize) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: isize) {
        self.vel_y = vel_y;
    }

    #[inline(always)]
    fn cond(&mut self) -> &mut Condition {
        &mut self.cond
    }

    #[inline(always)]
    fn flags(&mut self) -> &mut Flag {
        &mut self.flags
    }

    #[inline(always)]
    fn direction(&self) -> Direction {
        self.direction
    }

    #[inline(always)]
    fn is_player(&self) -> bool {
        true
    }

    fn player_left_pressed(&self) -> bool {
        self.controller.move_left()
    }

    fn player_right_pressed(&self) -> bool {
        self.controller.move_right()
    }
}

impl Player {
    fn judge_hit_npc_solid_soft(&mut self, npc: &NPC) -> Flag {
        let mut flags = Flag(0);

        if ((self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize - 3 * 0x200))
            && ((self.y + self.hit_bounds.top as isize) > (npc.y - npc.hit_bounds.bottom as isize + 3 * 0x200))
            && ((self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize))
            && ((self.x - self.hit_bounds.right as isize) > npc.x) {
            if self.vel_x < 0x200 {
                self.vel_x += 0x200;
            }

            flags.set_hit_left_wall(true);
        }

        if ((self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize - 3 * 0x200))
            && ((self.y + self.hit_bounds.top as isize) > (npc.y - npc.hit_bounds.bottom as isize + 3 * 0x200))
            && ((self.x + self.hit_bounds.right as isize - 0x200) > (npc.x - npc.hit_bounds.right as isize))
            && ((self.x + self.hit_bounds.right as isize - 0x200) < npc.x) {
            if self.vel_x > -0x200 {
                self.vel_x -= 0x200;
            }

            flags.set_hit_right_wall(true);
        }


        if ((self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize - 3 * 0x200))
            && ((self.x + self.hit_bounds.right as isize) > (npc.x - npc.hit_bounds.right as isize + 3 * 0x200))
            && ((self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize))
            && ((self.y - self.hit_bounds.top as isize) > npc.y) {
            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            flags.set_hit_top_wall(true);
        }

        if ((self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize - 3 * 0x200))
            && ((self.x + self.hit_bounds.right as isize) > (npc.x - npc.hit_bounds.right as isize + 3 * 0x200))
            && ((self.y + self.hit_bounds.bottom as isize - 0x200) > (npc.y - npc.hit_bounds.top as isize))
            && ((self.y + self.hit_bounds.bottom as isize - 0x200) < (npc.y + 3 * 0x200)) {
            if npc.npc_flags.bouncy() {
                self.vel_y = npc.vel_y - 0x200;
                flags.set_hit_bottom_wall(true);
            } else if !self.flags.hit_bottom_wall() && self.vel_y > npc.vel_y {
                self.y = npc.y - npc.hit_bounds.top as isize - self.hit_bounds.bottom as isize + 0x200;
                self.vel_y = npc.vel_y;
                self.x += npc.vel_x;
                flags.set_hit_bottom_wall(true);
            }
        }

        flags
    }

    fn judge_hit_npc_solid_hard(&mut self, npc: &NPC, state: &mut SharedGameState) -> Flag {
        let mut flags = Flag(0);

        let fx1 = abs(self.x - npc.x) as f32;
        let fy1 = abs(self.y - npc.y) as f32;

        let fx2 = npc.hit_bounds.right as f32;
        let fy2 = npc.hit_bounds.top as f32;

        let fx1 = if fx1 == 0.0 { 1.0 } else { fx1 };
        let fx2 = if fx2 == 0.0 { 1.0 } else { fx2 };

        if fy1 / fx1 <= fy2 / fx2 {
            if (self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize)
                && (self.y + self.hit_bounds.bottom as isize) > (npc.y - npc.hit_bounds.top as isize) {
                if (self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize)
                    && (self.x - self.hit_bounds.right as isize) > npc.x {
                    if self.vel_x < npc.vel_x {
                        self.vel_x = npc.vel_x;
                    }

                    self.x = npc.x + npc.hit_bounds.right as isize + self.hit_bounds.right as isize;
                    flags.set_hit_left_wall(true);
                }

                if (self.x + self.hit_bounds.right as isize) > (npc.x - npc.hit_bounds.right as isize)
                    && (self.x + self.hit_bounds.right as isize) < npc.x {
                    if self.vel_x > npc.vel_x {
                        self.vel_x = npc.vel_x;
                    }

                    self.x = npc.x - npc.hit_bounds.right as isize - self.hit_bounds.right as isize;
                    flags.set_hit_right_wall(true);
                }
            }
        } else if (self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize)
            && (self.x + self.hit_bounds.right as isize) > (npc.x - npc.hit_bounds.right as isize) {
            if (self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize)
                && (self.y - self.hit_bounds.top as isize) > npc.y {
                if self.vel_y >= npc.vel_y {
                    if self.vel_y < 0 {
                        self.vel_y = 0;
                    }
                } else {
                    self.y = npc.y + npc.hit_bounds.bottom as isize + self.hit_bounds.top as isize + 0x200;
                    self.vel_y = npc.vel_y;
                }

                flags.set_hit_top_wall(true);
            }

            if (self.y + self.hit_bounds.bottom as isize) > (npc.y - npc.hit_bounds.top as isize)
                && (self.y + self.hit_bounds.bottom as isize) < (npc.y + 3 * 0x200) {
                if self.vel_y - npc.vel_y > 2 * 0x200 {
                    state.sound_manager.play_sfx(23);
                }

                if self.control_mode == ControlMode::IronHead {
                    self.y = npc.y - npc.hit_bounds.top as isize - self.hit_bounds.bottom as isize + 0x200;
                    flags.set_hit_bottom_wall(true);
                } else if npc.npc_flags.bouncy() {
                    self.vel_y = npc.vel_y - 0x200;
                    flags.set_hit_bottom_wall(true);
                } else if !self.flags.hit_bottom_wall() && self.vel_y > npc.vel_y {
                    self.y = npc.y - npc.hit_bounds.top as isize - self.hit_bounds.bottom as isize + 0x200;
                    self.vel_y = npc.vel_y;
                    self.x += npc.vel_x;

                    flags.set_hit_bottom_wall(true);
                }
            }
        }

        flags
    }

    fn judge_hit_npc_non_solid(&mut self, npc: &NPC) -> Flag {
        let mut flags = Flag(0);
        let hit_left = if npc.direction == Direction::Left { npc.hit_bounds.left } else { npc.hit_bounds.right } as isize;
        let hit_right = if npc.direction == Direction::Left { npc.hit_bounds.right } else { npc.hit_bounds.left } as isize;

        if self.x + (2 * 0x200) > npc.x - hit_left
            && self.x - (2 * 0x200) < npc.x + hit_right
            && self.y + (2 * 0x200) > npc.y - npc.hit_bounds.top as isize
            && self.y - (2 * 0x200) < npc.y + npc.hit_bounds.bottom as isize {
            flags.set_hit_left_wall(true);
        }

        flags
    }

    fn tick_npc_collision(&mut self, id: TargetPlayer, state: &mut SharedGameState, npc: &mut NPC, inventory: &mut Inventory) {
        let flags: Flag;

        if npc.npc_flags.solid_soft() {
            flags = self.judge_hit_npc_solid_soft(npc.borrow());
            self.flags.0 |= flags.0;
        } else if npc.npc_flags.solid_hard() {
            flags = self.judge_hit_npc_solid_hard(npc.borrow(), state);
            self.flags.0 |= flags.0;
        } else {
            flags = self.judge_hit_npc_non_solid(npc.borrow());
        }

        if !npc.cond.drs_boss() && flags.0 != 0 {
            match npc.npc_type {
                // experience pickup
                1 => {
                    state.sound_manager.play_sfx(14);
                    match inventory.add_xp(npc.exp, state) {
                        AddExperienceResult::None => {}
                        AddExperienceResult::LevelUp => {
                            state.sound_manager.play_sfx(27);
                            state.create_caret(self.x, self.y, CaretType::LevelUp, Direction::Left);
                        }
                        AddExperienceResult::AddStar => {
                            if self.equip.has_whimsical_star() && self.stars < 3 {
                                self.stars += 1;
                            }
                        }
                    }
                    npc.cond.set_alive(false);
                }
                // missile pickup
                86 => {
                    // todo add bullets
                    npc.cond.set_alive(false);

                    state.sound_manager.play_sfx(42);
                }
                // heart pickup
                87 => {
                    self.life = self.max_life.min(self.life.saturating_add(npc.exp));
                    npc.cond.set_alive(false);

                    state.sound_manager.play_sfx(20);
                }
                _ => {}
            }
        }

        if npc.npc_flags.interactable() && !state.control_flags.interactions_disabled() && flags.0 != 0 && self.cond.interacted() {
            state.control_flags.set_tick_world(true);
            state.control_flags.set_interactions_disabled(true);
            state.textscript_vm.executor_player = id;
            state.textscript_vm.start_script(npc.event_num);
            self.cond.set_interacted(false);
            self.vel_x = 0;
            self.question = false;
        }

        if npc.npc_flags.event_when_touched() && !state.control_flags.interactions_disabled() && flags.0 != 0 {
            state.control_flags.set_tick_world(true);
            state.control_flags.set_interactions_disabled(true);
            state.textscript_vm.executor_player = id;
            state.textscript_vm.start_script(npc.event_num);
        }

        if state.control_flags.control_enabled() && !npc.npc_flags.interactable() {
            if npc.npc_flags.rear_and_top_not_hurt() {
                if flags.hit_left_wall() && npc.vel_x > 0
                    || flags.hit_right_wall() && npc.vel_x < 0
                    || flags.hit_top_wall() && npc.vel_y > 0
                    || flags.hit_bottom_wall() && npc.vel_y < 0 {
                    self.damage(npc.damage as isize, state);
                }
            } else if flags.0 != 0 && npc.damage != 0 && !state.control_flags.interactions_disabled() {
                self.damage(npc.damage as isize, state);
            }
        }
    }

    pub fn tick_npc_collisions(&mut self, id: TargetPlayer, state: &mut SharedGameState, npc_map: &mut NPCMap, inventory: &mut Inventory) {
        if !self.cond.alive() {
            return;
        }

        for npc_cell in npc_map.npcs.values() {
            let mut npc = npc_cell.borrow_mut();
            if !npc.cond.alive() { continue; }

            self.tick_npc_collision(id, state, npc.borrow_mut(), inventory);
        }

        for boss_npc in npc_map.boss_map.parts.iter_mut() {
            if boss_npc.cond.alive() {
                self.tick_npc_collision(id, state, boss_npc, inventory);
            }
        }

        if self.question {
            state.create_caret(self.x, self.y, CaretType::QuestionMark, Direction::Left);
        }
    }
}
