use ggez::GameResult;

use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;
use crate::common::Direction;

impl NPC {
    pub(crate) fn tick_n113_professor_booster(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.animate(5, 2, 5);

                self.x += self.direction.vector_x() * 0x200;
            }
            5 => {
                self.anim_num = 6;
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.hit_bounds.bottom = 16 * 0x200;
                    self.x -= 16 * 0x200;
                    self.y += 8 * 0x200;
                }

                self.action_counter += 1;
                if self.action_counter == 64 {
                    self.action_num = 32;
                    self.action_counter = 0;
                }
            }
            32 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 33;
                    self.anim_num = 1;
                    self.hit_bounds.bottom = 8 * 0x200;
                }
            }
            33 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 34;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };

        self.anim_rect = state.constants.npc.n113_professor_booster[self.anim_num as usize + dir_offset];

        if self.action_num == 31 {
            self.anim_rect.bottom = self.action_counter / 4 + self.anim_rect.top;
            if self.action_counter / 2 % 2 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }
}
