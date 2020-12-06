use crate::caret::CaretType;
use crate::common::{Direction, Rect, CDEG_RAD};
use ggez::GameResult;
use crate::npc::{NPC, NPCMap};
use crate::npc::boss::BossNPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n108_balfrog_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_counter > 300 || (self.flags.0 & 0xff) != 0 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.action_counter += 1;
        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 2 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n108_balfrog_projectile[self.anim_num as usize];

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b02_balfrog(&mut self, state: &mut SharedGameState, player: &Player) {
        match self.parts[0].action_num {
            0 => {
                self.hurt_sound[0] = 52;
                self.parts[0].x = 6 * 16 * 0x200;
                self.parts[0].y = 12 * 16 * 0x200;
                self.parts[0].direction = Direction::Right;
                self.parts[0].display_bounds = Rect {
                    left: 48 * 0x200,
                    top: 48 * 0x200,
                    right: 32 * 0x200,
                    bottom: 16 * 0x200,
                };
                self.parts[0].hit_bounds = Rect {
                    left: 24 * 0x200,
                    top: 16 * 0x200,
                    right: 24 * 0x200,
                    bottom: 16 * 0x200,
                };
                self.parts[0].size = 3;
                self.parts[0].exp = 1;
                self.parts[0].event_num = 1000;
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].life = 300;
            }
            10 => {
                self.parts[0].action_num = 11;
                self.parts[0].anim_num = 3;
                self.parts[0].cond.set_alive(true);
                self.parts[0].anim_rect = state.constants.npc.b02_balfrog[9];

                self.parts[1].cond.set_alive(true);
                self.parts[1].cond.set_damage_boss(true);
                self.parts[1].damage = 5;

                self.parts[2].cond.set_alive(true);
                self.parts[2].damage = 5;

                let mut npc = NPCMap::create_npc(4, &state.npc_table);

                for _ in 0..8 {
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-12..12) as isize * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-12..12) as isize * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                    npc.vel_y = self.parts[0].rng.range(-0x600..0) as isize;

                    state.new_npcs.push(npc);
                }
            }
            20 | 21 => {
                if self.parts[0].action_num == 20 {
                    self.parts[0].action_num = 21;
                    self.parts[0].action_counter = 0
                }

                self.parts[0].action_counter += 1;
                if (self.parts[0].action_counter / 2 % 2) != 0 {
                    self.parts[0].anim_num = 3;
                } else {
                    self.parts[0].anim_num = 0;
                }
            }
            100 | 101 => {
                if self.parts[0].action_num == 100 {
                    self.parts[0].action_num = 101;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_num = 1;
                    self.parts[0].vel_x = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_num = 102;
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num = 2;
                }
            }
            102 => {
                self.parts[0].anim_counter += 1;

                if self.parts[0].anim_counter > 10 {
                    self.parts[0].action_num = 103;
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num = 1;
                }
            }
            103 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 4 {
                    self.parts[0].action_num = 104;
                    self.parts[0].anim_num = 5;
                    self.parts[0].vel_x = self.parts[0].direction.vector_x() * 0x200;
                    self.parts[0].vel_y = -2 * 0x200;
                    self.parts[0].display_bounds.top = 64 * 0x200;
                    self.parts[0].display_bounds.bottom = 24 * 0x200;

                    state.sound_manager.play_sfx(25);
                }
            }
            104 => {
                if self.parts[0].direction == Direction::Left && self.parts[0].flags.hit_left_wall() {
                    self.parts[0].direction = Direction::Right;
                    self.parts[0].vel_x = 0x200;
                }

                if self.parts[0].direction == Direction::Right && self.parts[0].flags.hit_right_wall() {
                    self.parts[0].direction = Direction::Left;
                    self.parts[0].vel_x = -0x200;
                }

                if self.parts[0].flags.hit_bottom_wall() {
                    self.parts[0].action_num = 100;
                    self.parts[0].anim_num = 1;
                    self.parts[0].display_bounds.top = 48 * 0x200;
                    self.parts[0].display_bounds.bottom = 16 * 0x200;

                    if self.parts[0].direction == Direction::Left && self.parts[0].x < player.x {
                        self.parts[0].direction = Direction::Right;
                        self.parts[0].action_num = 110;
                    }

                    if self.parts[0].direction == Direction::Right && self.parts[0].x > player.x {
                        self.parts[0].direction = Direction::Left;
                        self.parts[0].action_num = 110;
                    }

                    let mut npc = NPCMap::create_npc(110, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].rng.range(4..16) as isize * 16 * 0x200;
                    npc.y = self.parts[0].rng.range(0..4) as isize * 16 * 0x200;
                    npc.direction = Direction::FacingPlayer;

                    state.new_npcs.push(npc);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);

                    for _ in 0..4 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-12..12) as isize * 0x200;
                        npc.y = self.parts[0].y + self.parts[0].rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                        npc.vel_y = self.parts[0].rng.range(-0x600..0) as isize;

                        state.new_npcs.push(npc);
                    }

                    state.quake_counter = 30;
                    state.sound_manager.play_sfx(26);
                }
            }
            110 | 111 => {
                if self.parts[0].action_num == 110 {
                    self.parts[0].anim_num = 1;
                    self.parts[0].action_num = 111;
                    self.parts[0].action_counter = 0;
                }

                self.parts[0].action_counter += 1;

                self.parts[0].vel_x = self.parts[0].vel_x * 8 / 9;

                if self.parts[0].action_counter > 50 {
                    self.parts[0].anim_num = 2;
                    self.parts[0].anim_counter = 0;
                    self.parts[0].action_num = 112;
                }
            }
            112 => {
                self.parts[0].anim_counter += 1;

                if self.parts[0].anim_counter > 4 {
                    self.parts[0].action_num = 113;
                    self.parts[0].action_counter = 0;
                    self.parts[0].vel_x2 = 16;
                    self.parts[0].anim_num = 3;
                    self.parts[0].target_x = self.parts[0].life as isize;
                    self.parts[1].npc_flags.set_shootable(true);
                }
            }
            113 => {
                if self.parts[0].shock != 0 {
                    if self.parts[0].action_counter2 / 2 % 2 != 0 {
                        self.parts[0].anim_num = 4;
                    } else {
                        self.parts[0].anim_num = 3;
                    }
                } else {
                    self.parts[0].action_counter2 = 0;
                    self.parts[0].anim_num = 3;
                }

                self.parts[0].vel_x = self.parts[0].vel_x * 10 / 11;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 16 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].vel_x2 = self.parts[0].vel_x2.saturating_sub(1);

                    let px = self.parts[0].x + self.parts[0].direction.vector_x() * 2 * 16 * 0x200 - player.x;
                    let py = self.parts[0].y - 8 * 0x200 - player.y;

                    let deg = f64::atan2(py as f64, px as f64)
                        + self.parts[0].rng.range(-16..16) as f64 * CDEG_RAD;
                    // todo rand

                    let mut npc = NPCMap::create_npc(108, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].direction.vector_x() * 2 * 16 * 0x200;
                    npc.y = self.parts[0].y - 8 * 0x200;
                    npc.vel_x = (deg.cos() * -512.0) as isize;
                    npc.vel_y = (deg.sin() * -512.0) as isize;

                    state.new_npcs.push(npc);

                    state.sound_manager.play_sfx(39);

                    if self.parts[0].vel_x2 == 0 || (self.parts[0].life as isize) < self.parts[0].target_x - 90 {
                        self.parts[0].action_num = 114;
                        self.parts[0].action_counter = 0;
                        self.parts[0].anim_num = 2;
                        self.parts[0].anim_counter = 0;
                        self.parts[1].npc_flags.set_shootable(false);
                    }
                }
            }
            114 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 10 {
                    self.parts[0].anim_num = 1;
                    self.parts[0].anim_counter = 0;

                    self.parts[1].action_counter2 += 1;
                    if self.parts[1].action_counter2 > 2 {
                        self.parts[1].action_counter2 = 0;
                        self.parts[0].action_num = 120;
                    } else {
                        self.parts[0].action_num = 100;
                    }
                }
            }
            120 | 121 => {
                if self.parts[0].action_num == 120 {
                    self.parts[0].action_num = 121;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_num = 1;
                    self.parts[0].vel_x = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_num = 122;
                    self.parts[0].anim_num = 2;
                    self.parts[0].anim_counter = 0;
                }
            }
            122 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 20 {
                    self.parts[0].action_num = 123;
                    self.parts[0].anim_num = 1;
                    self.parts[0].anim_counter = 0;
                }
            }
            123 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 4 {
                    self.parts[0].action_num = 124;
                    self.parts[0].anim_num = 5;
                    self.parts[0].vel_x = -5 * 0x200;
                    self.parts[0].display_bounds.top = 64 * 0x200;
                    self.parts[0].display_bounds.bottom = 24 * 0x200;

                    state.sound_manager.play_sfx(25);
                }
            }
            124 => {
                if self.parts[0].flags.hit_bottom_wall() {
                    self.parts[0].action_num = 100;
                    self.parts[0].anim_num = 1;
                    self.parts[0].display_bounds.top = 48 * 0x200;
                    self.parts[0].display_bounds.bottom = 16 * 0x200;

                    let mut npc = NPCMap::create_npc(104, &state.npc_table);
                    for _ in 0..2 {
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].rng.range(4..16) as isize * 16 * 0x200;
                        npc.y = self.parts[0].rng.range(0..4) as isize * 16 * 0x200;
                        npc.direction = Direction::FacingPlayer;

                        state.new_npcs.push(npc);
                    }

                    let mut npc = NPCMap::create_npc(110, &state.npc_table);
                    for _ in 0..6 {
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].rng.range(4..16) as isize * 16 * 0x200;
                        npc.y = self.parts[0].rng.range(0..4) as isize * 16 * 0x200;
                        npc.direction = Direction::FacingPlayer;

                        state.new_npcs.push(npc);
                    }

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    for _ in 0..8 {
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-12..12) as isize * 0x200;
                        npc.y = self.parts[0].y + self.parts[0].hit_bounds.bottom as isize;
                        npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                        npc.vel_y = self.parts[0].rng.range(-0x600..0) as isize;
                        npc.direction = Direction::Left;

                        state.new_npcs.push(npc);
                    }

                    if self.parts[0].direction == Direction::Left && self.parts[0].x < player.x {
                        self.parts[0].action_num = 110;
                        self.parts[0].direction = Direction::Right;
                    }

                    if self.parts[0].direction == Direction::Right && self.parts[0].x > player.x {
                        self.parts[0].action_num = 110;
                        self.parts[0].direction = Direction::Right;
                    }

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 60;
                }
            }
            130 | 131 => {
                if self.parts[0].action_num == 130 {
                    self.parts[0].action_num = 131;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_num = 3;
                    self.parts[0].vel_x = 0;

                    self.parts[1].cond.set_alive(false);
                    self.parts[2].cond.set_alive(false);

                    state.sound_manager.play_sfx(72);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    for _ in 0..8 {
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-12..12) as isize * 0x200;
                        npc.y = self.parts[0].y + self.parts[0].rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                        npc.vel_y = self.parts[0].rng.range(-0x600..0) as isize;
                        npc.direction = Direction::Left;
                        state.new_npcs.push(npc);
                    }
                }

                self.parts[0].action_counter += 1;
                if (self.parts[0].action_counter % 5) == 0 {
                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-12..12) as isize * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-12..12) as isize * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                    npc.vel_y = self.parts[0].rng.range(-0x600..0) as isize;
                    npc.direction = Direction::Left;
                    state.new_npcs.push(npc);
                }

                self.parts[0].x += if (self.parts[0].action_counter / 2 % 2) != 0 {
                    -0x200
                } else {
                    0x200
                };

                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_num = 132;
                    self.parts[0].action_counter = 0;
                }
            }
            132 => {
                self.parts[0].action_counter += 1;
                if (self.parts[0].action_counter / 2 % 2) != 0 {
                    self.parts[0].anim_num = 6;
                    self.parts[0].display_bounds = Rect {
                        left: 20 * 0x200,
                        top: 12 * 0x200,
                        right: 20 * 0x200,
                        bottom: 12 * 0x200,
                    };
                } else {
                    self.parts[0].anim_num = 3;
                    self.parts[0].display_bounds = Rect {
                        left: 48 * 0x200,
                        top: 48 * 0x200,
                        right: 32 * 0x200,
                        bottom: 16 * 0x200,
                    };
                }

                if (self.parts[0].action_counter % 9) == 0 {
                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-12..12) as isize * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-12..12) as isize * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                    npc.vel_y = self.parts[0].rng.range(-0x600..0) as isize;
                    npc.direction = Direction::Left;
                    state.new_npcs.push(npc);
                }

                if self.parts[0].action_counter > 150 {
                    self.parts[0].action_num = 140;
                    self.parts[0].hit_bounds.bottom = 12 * 0x200;
                }
            }
            140 | 141 => {
                if self.parts[0].action_num == 140 {
                    self.parts[0].action_num = 141;
                }

                if self.parts[0].flags.hit_bottom_wall() {
                    self.parts[0].action_num = 142;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_num = 7;
                }
            }
            142 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 30 {
                    self.parts[0].anim_num = 8;
                    self.parts[0].vel_y = -5 * 0x200;
                    self.parts[0].npc_flags.set_ignore_solidity(true);
                    self.parts[0].action_num = 143;
                }
            }
            143 => {
                self.parts[0].vel_y = -5 * 0x200;
                if self.parts[0].y < 0 {
                    self.parts[0].cond.set_alive(false);

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;
                }
            }
            _ => {}
        }

        self.parts[0].vel_y += 0x40;
        if self.parts[0].vel_y > 0x5ff {
            self.parts[0].vel_y = 0x5ff;
        }

        self.parts[0].x += self.parts[0].vel_x;
        self.parts[0].y += self.parts[0].vel_y;

        let dir_offset = if self.parts[0].direction == Direction::Left { 0 } else { 9 };
        self.parts[0].anim_rect = state.constants.npc.b02_balfrog[self.parts[0].anim_num as usize + dir_offset];

        match self.parts[0].anim_num {
            0 => {
                self.hurt_sound[1] = 52;
                self.parts[1].size = 3;
                self.parts[1].npc_flags.set_invulnerable(true);
                self.parts[1].hit_bounds = Rect {
                    left: 16 * 0x200,
                    top: 16 * 0x200,
                    right: 16 * 0x200,
                    bottom: 16 * 0x200,
                };

                self.hurt_sound[2] = 52;
                self.parts[2].size = 3;
                self.parts[2].npc_flags.set_invulnerable(true);
                self.parts[2].hit_bounds = Rect {
                    left: 24 * 0x200,
                    top: 16 * 0x200,
                    right: 24 * 0x200,
                    bottom: 16 * 0x200,
                };
            }
            1 => {
                self.parts[1].x = self.parts[0].x + self.parts[0].direction.vector_x() * 24 * 0x200;
                self.parts[1].y = self.parts[0].y - 24 * 0x200;

                self.parts[2].x = self.parts[0].x;
                self.parts[2].y = self.parts[0].y;
            }
            2 => {
                self.parts[1].x = self.parts[0].x + self.parts[0].direction.vector_x() * 24 * 0x200;
                self.parts[1].y = self.parts[0].y - 20 * 0x200;

                self.parts[2].x = self.parts[0].x;
                self.parts[2].y = self.parts[0].y;
            }
            3 | 4 => {
                self.parts[1].x = self.parts[0].x + self.parts[0].direction.vector_x() * 24 * 0x200;
                self.parts[1].y = self.parts[0].y - 16 * 0x200;

                self.parts[2].x = self.parts[0].x;
                self.parts[2].y = self.parts[0].y;
            }
            5 => {
                self.parts[1].x = self.parts[0].x + self.parts[0].direction.vector_x() * 24 * 0x200;
                self.parts[1].y = self.parts[0].y - 43 * 0x200;

                self.parts[2].x = self.parts[0].x;
                self.parts[2].y = self.parts[0].y;
            }
            _ => { }
        }
    }
}
