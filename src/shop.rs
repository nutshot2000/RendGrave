// The Ashford merchant: buy potions and gear, sell what you don't need. The stock is
// rolled fresh each time you come back to town, around your level, never plain junk.
use crate::audio::{Sfx, Snd};
use crate::gear::BAG_SIZE;
use crate::game::*;
use crate::items::*;
use crate::rng::*;

pub const STOCK: usize = 8;
pub const MAX_POTIONS: i32 = 9;

impl Game {
    pub fn restock(&mut self) {
        let lv = self.player.sk.level.max(2);
        self.shop.clear();
        while self.shop.len() < STOCK {
            let it = roll(lv + ri(3) as i32 - 1, 0.12);
            if it.rarity == Rarity::Common || it.rarity == Rarity::Legendary {
                continue;
            }
            self.shop.push(it);
        }
    }
    pub fn price(it: &Item) -> i32 {
        it.value() * 4
    }
    pub fn potion_price(&self) -> i32 {
        20 + 4 * self.player.sk.level
    }

    pub fn update_shop(&mut self, inp: &Input) {
        self.shop_hover = crate::hud::shop_at(inp.mouse, self.view);
        let click = inp.lmb && !self.prev.lmb;
        let rclick = inp.rmb && !self.prev.rmb;
        let (x, y) = (self.player.x, self.player.y);
        match self.shop_hover {
            Some((0, k)) if click && k < self.shop.len() => {
                let cost = Game::price(&self.shop[k]);
                if self.player.bag.len() >= BAG_SIZE {
                    self.toast = Some(("YOUR BAG IS FULL".into(), 90));
                } else if self.player.gold < cost {
                    self.toast = Some((format!("YOU NEED {} GOLD", cost), 90));
                } else {
                    self.player.gold -= cost;
                    let it = self.shop.remove(k);
                    self.toast = Some((format!("BOUGHT {}", it.name), 120));
                    self.player.bag.push(it);
                    self.sounds.push(Snd { k: Sfx::Coin, x, y, v: 1.0 });
                    self.save();
                }
            }
            Some((2, _)) if click => {
                let cost = self.potion_price();
                if self.player.potions >= MAX_POTIONS {
                    self.toast = Some((format!("YOU CAN CARRY {} POTIONS", MAX_POTIONS), 90));
                } else if self.player.gold < cost {
                    self.toast = Some((format!("YOU NEED {} GOLD", cost), 90));
                } else {
                    self.player.gold -= cost;
                    self.player.potions += 1;
                    self.sounds.push(Snd { k: Sfx::Potion, x, y, v: 0.8 });
                    self.save();
                }
            }
            Some((1, i)) if click && i < self.player.bag.len() => self.equip_from_bag(i),
            Some((1, i)) if rclick && i < self.player.bag.len() => {
                let it = self.player.bag.remove(i);
                let v = it.value();
                self.player.gold += v;
                self.toast = Some((format!("SOLD {} FOR {} GOLD", it.name, v), 120));
                self.sounds.push(Snd { k: Sfx::Coin, x, y, v: 0.9 });
                self.save();
            }
            _ => {}
        }
    }
}
