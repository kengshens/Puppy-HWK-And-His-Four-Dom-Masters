use std::time::Instant;
use macroquad::prelude::*;
use ::rand::prelude::*;
use mysql::*;
use mysql::prelude::*;

// ==================== 基础类型定义 ====================



/// 2D向量结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn distance(&self, other: &Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    pub fn normalize(&self) -> Vec2 {
        let length = (self.x.powi(2) + self.y.powi(2)).sqrt();
        if length > 0.0 {
            Vec2::new(self.x / length, self.y / length)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

// ==================== 游戏状态枚举 ====================

/// 游戏状态
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    MainMenu,
    WeaponSelect,
    Login,
    Battle,
    RogueSelection,
    GameOver,
}

/// 输入模式
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    None,
    Username,
    Password,
}

// ==================== 武器系统 ====================

/// 武器类型
#[derive(Debug, Clone, PartialEq)]
pub enum WeaponType {
    MachineGun,
    Laser,
    Shotgun,
}

/// 武器结构
#[derive(Debug, Clone)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub attack_power: i32,
    pub attack_speed: f32,
    pub bullet_speed: f32,
    pub bullet_count: i32,
    pub enhancement_level: i32,
}

impl Weapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        let (attack_power, attack_speed, bullet_speed, bullet_count) = match weapon_type {
            WeaponType::MachineGun => (2, 1.2, 2.0, 2),
            WeaponType::Laser => (4, 1.25, 0.0, 1),
            WeaponType::Shotgun => (4, 1.0, 3.0, 3),
        };
        
        Self {
            weapon_type,
            attack_power,
            attack_speed,
            bullet_speed,
            bullet_count,
            enhancement_level: 0,
        }
    }
    
    pub fn get_total_attack_power(&self) -> i32 {
        self.attack_power + self.enhancement_level
    }
}

// ==================== 敌人系统 ====================

/// 敌人类型
#[derive(Debug, Clone, PartialEq)]
pub enum EnemyType {
    Scout,   // 侦察机
    Heavy,   // 重甲舰
    Carrier, // 航母
    Boss,    // BOSS
}

/// 敌人结构
#[derive(Debug, Clone)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub health: i32,
    pub max_health: i32,
    pub bullet_damage: i32,
    pub collision_damage: i32,
    pub last_shot_time: Instant,
    pub spawn_time: Instant,
    pub special_state: i32,
    pub is_invincible: bool,
    pub shield_health: i32,
    pub movement_pattern: i32,
    pub movement_timer: f32,
    pub target_position: Vec2,
    pub has_reached_zone: bool,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType, position: Vec2) -> Self {
        let mut rng = ::rand::thread_rng();
        
        let (health, velocity, bullet_damage, collision_damage, movement_pattern, target_position) = match enemy_type {
            EnemyType::Scout => (20, Vec2::new(0.0, 0.5), 0, 5, 0, Vec2::new(0.0, 0.0)),
            EnemyType::Heavy => (30, Vec2::new(0.0, 0.8), 2, 5, rng.gen_range(1..=4), Vec2::new(position.x, 120.0)),
            EnemyType::Carrier => (100, Vec2::new(0.0, 0.3), 0, 10, 0, Vec2::new(0.0, 0.0)),
            EnemyType::Boss => (150, Vec2::new(0.0, 0.5), 10, 20, 1, Vec2::new(position.x, 100.0)),
        };
        
        let special_state = match enemy_type {
            EnemyType::Boss => 1,
            _ => 0,
        };
        
        Self {
            enemy_type,
            position,
            velocity,
            health,
            max_health: health,
            bullet_damage,
            collision_damage,
            last_shot_time: Instant::now(),
            spawn_time: Instant::now(),
            special_state,
            is_invincible: false,
            shield_health: 0,
            movement_pattern,
            movement_timer: 0.0,
            target_position,
            has_reached_zone: false,
        }
    }
    
    pub fn get_drop_gold(&self) -> i32 {
        match self.enemy_type {
            EnemyType::Scout => 10,
            EnemyType::Heavy => 20,
            EnemyType::Carrier => 50,
            EnemyType::Boss => 100,
        }
    }
    
    pub fn get_drop_exp(&self) -> i32 {
        match self.enemy_type {
            EnemyType::Scout => 20,
            EnemyType::Heavy => 30,
            EnemyType::Carrier => 50,
            EnemyType::Boss => 0,
        }
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        if self.is_invincible {
            return;
        }
        
        if self.shield_health > 0 {
            self.shield_health = (self.shield_health - damage).max(0);
        } else {
            self.health = (self.health - damage).max(0);
        }
        
        // Boss进入第二阶段
        if self.enemy_type == EnemyType::Boss && self.health <= 75 && self.special_state == 1 {
            self.special_state = 2;
            self.is_invincible = true;
        }
    }
}

// ==================== 子弹系统 ====================

/// 子弹类型
#[derive(Debug, Clone, PartialEq)]
pub enum BulletType {
    PlayerMachineGun,
    PlayerLaser,
    PlayerShotgun,
    EnemyHeavy,
    EnemyBoss,
    EnemyGeneric,
}

/// 子弹结构
#[derive(Debug, Clone)]
pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub damage: i32,
    pub is_player_bullet: bool,
    pub piercing_count: i32,
    pub ricochet_count: i32,
    pub burning_damage: i32,
    pub explosion_damage: f32,
    pub is_crit: bool,
    pub hit_enemies: Vec<usize>,
    pub bullet_type: BulletType,
}

impl Bullet {
    pub fn new(position: Vec2, velocity: Vec2, damage: i32, is_player_bullet: bool, bullet_type: BulletType) -> Self {
        Self {
            position,
            velocity,
            damage,
            is_player_bullet,
            piercing_count: 0,
            ricochet_count: 0,
            burning_damage: 0,
            explosion_damage: 0.0,
            is_crit: false,
            hit_enemies: Vec::new(),
            bullet_type,
        }
    }
}

// ==================== 道具系统 ====================

/// 道具类型
#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    HealthPack,
}

/// 道具结构
#[derive(Debug, Clone)]
pub struct Item {
    pub item_type: ItemType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub value: i32,
    pub spawn_time: Instant,
}

impl Item {
    pub fn new(item_type: ItemType, position: Vec2, value: i32) -> Self {
        Self {
            item_type,
            position,
            velocity: Vec2::new(0.0, 1.0),
            value,
            spawn_time: Instant::now(),
        }
    }
}

// ==================== 肉鸽升级系统 ====================

/// 升级稀有度
#[derive(Debug, Clone, PartialEq)]
pub enum UpgradeRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

/// 肉鸽升级
#[derive(Debug, Clone)]
pub struct RogueUpgrade {
    pub id: u32,
    pub name: String,
    pub short_desc: String,
    pub detailed_desc: String,
    pub icon: String,
    pub max_selections: Option<u32>,
    pub current_selections: u32,
    pub rarity: UpgradeRarity,
}

impl RogueUpgrade {
    pub fn new(id: u32, name: &str, short_desc: &str, detailed_desc: &str, icon: &str, rarity: UpgradeRarity, max_selections: Option<u32>) -> Self {
        Self {
            id,
            name: name.to_string(),
            short_desc: short_desc.to_string(),
            detailed_desc: detailed_desc.to_string(),
            icon: icon.to_string(),
            max_selections,
            current_selections: 0,
            rarity,
        }
    }
    
    pub fn get_rarity_color(&self) -> Color {
        match self.rarity {
            UpgradeRarity::Common => WHITE,
            UpgradeRarity::Rare => Color::new(0.3, 0.6, 1.0, 1.0),
            UpgradeRarity::Epic => Color::new(0.8, 0.3, 1.0, 1.0),
            UpgradeRarity::Legendary => Color::new(1.0, 0.8, 0.2, 1.0),
        }
    }
}

// ==================== 玩家系统 ====================

/// 玩家结构
#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec2,
    pub health: i32,
    pub max_health: i32,
    pub level: i32,
    pub experience: i32,
    pub experience_needed: i32,
    pub weapon: Weapon,
    pub last_shot_time: Instant,
    pub attack_power_bonus: i32,
    pub crit_rate: f32,
    pub crit_damage: f32,
    pub bullet_count_bonus: i32,
    pub piercing: i32,
    pub ricochet: i32,
    pub burning_damage: i32,
    pub explosion_damage: f32,
    pub damage_reduction: i32,
    pub bullet_speed_bonus: f32,
    pub rogue_upgrades: Vec<RogueUpgrade>,
    pub last_damage_time: Instant,
    pub invincibility_duration: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(400.0, 550.0),
            health: 20,
            max_health: 20,
            level: 1,
            experience: 0,
            experience_needed: 100,
            weapon: Weapon::new(WeaponType::MachineGun),
            last_shot_time: Instant::now(),
            attack_power_bonus: 0,
            crit_rate: 0.0,
            crit_damage: 1.5,
            bullet_count_bonus: 0,
            piercing: 0,
            ricochet: 0,
            burning_damage: 0,
            explosion_damage: 0.0,
            damage_reduction: 0,
            bullet_speed_bonus: 0.0,
            rogue_upgrades: Vec::new(),
            last_damage_time: Instant::now(),
            invincibility_duration: 0.0,
        }
    }
    
    pub fn add_experience(&mut self, exp: i32) {
        self.experience += exp;
    }
    
    pub fn level_up(&mut self) {
        self.experience -= self.experience_needed;
        self.level += 1;
        self.experience_needed = 100 * self.level;
    }
    
    pub fn can_shoot(&self) -> bool {
        self.last_shot_time.elapsed().as_secs_f32() >= 1.0 / self.weapon.attack_speed
    }
    
    pub fn get_total_attack_power(&self) -> i32 {
        self.weapon.get_total_attack_power() + self.attack_power_bonus
    }
    
    pub fn get_total_bullet_count(&self) -> i32 {
        (self.weapon.bullet_count + self.bullet_count_bonus).min(self.weapon.bullet_count + 5)
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        if self.last_damage_time.elapsed().as_secs_f32() < self.invincibility_duration {
            return;
        }
        
        let actual_damage = (damage - self.damage_reduction).max(1);
        self.health = (self.health - actual_damage).max(0);
        
        self.last_damage_time = Instant::now();
        self.invincibility_duration = 1.0;
    }
}

// ==================== 游戏结算系统 ====================

/// 游戏结算结构
#[derive(Debug, Clone)]
pub struct GameResult {
    pub victory: bool,
    pub final_level: i32,
    pub coins_earned: i32,
    pub experience_gained: i32,
    pub survival_time: f32,
    pub enemies_defeated: i32,
    pub total_damage_dealt: i32,
    pub weapon_used: WeaponType,
}

impl GameResult {
    pub fn new(player: &Player, victory: bool, time: f32, enemies_defeated: i32, total_damage: i32) -> Self {
        Self {
            victory,
            final_level: player.level,
            coins_earned: 0,
            experience_gained: 0,
            survival_time: time,
            enemies_defeated,
            total_damage_dealt: total_damage,
            weapon_used: player.weapon.weapon_type.clone(),
        }
    }
}

// ==================== 用户系统 ====================

/// 用户数据
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
    pub is_logged_in: bool,
}

impl User {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            is_logged_in: false,
        }
    }

    // 传入 MySQL 连接池和用户输入，验证登录
    pub fn login(&mut self, pool: &Pool, username: &str, password: &str) -> Result<bool> {
        let mut conn = pool.get_conn()?;

        // 查询数据库，验证用户名密码是否匹配
        let result: Option<String> = conn.exec_first(
            "SELECT password FROM users WHERE username = :username",
            params! {
                "username" => username,
            },
        )?;

        if let Some(stored_password) = result {
            if stored_password == password {
                self.username = username.to_string();
                self.password = password.to_string();
                self.is_logged_in = true;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

// ==================== 主游戏结构 ====================

/// 游戏主结构
pub struct Game {
    // 核心状态
    pub state: GameState,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub items: Vec<Item>,
    
    // 时间相关
    pub start_time: Instant,
    pub last_spawn_time: Instant,
    
    // 游戏数据
    pub coins: i32,
    pub wins: i32,
    pub available_upgrades: Vec<RogueUpgrade>,
    pub screen_width: f32,
    pub screen_height: f32,
    pub rng: ThreadRng,
    
    // 用户系统
    pub user: User,
    pub pool: mysql::Pool,  // 修改为具体类型
    pub input_text: String,
    pub input_mode: InputMode,
    
    // 本局统计
    pub current_session_coins: i32,
    pub current_session_exp: i32,
    pub enemies_defeated_this_session: i32,
    pub total_damage_dealt: i32,
    pub game_result: Option<GameResult>,
    
    // 肉鸽升级相关
    pub rogue_selection_timer: f32,
    pub current_rogue_options: Vec<RogueUpgrade>,
    pub rogue_auto_selected: bool,
    pub rogue_auto_selected_timer: f32,
    
    // 纹理资源
    pub player_texture: Option<Texture2D>,
    pub scout_texture: Option<Texture2D>,
    pub heavy_texture: Option<Texture2D>,
    pub carrier_texture: Option<Texture2D>,
    pub boss_texture: Option<Texture2D>,
    pub machinegun_bullet_texture: Option<Texture2D>,
    pub laser_bullet_texture: Option<Texture2D>,
    pub shotgun_bullet_texture: Option<Texture2D>,
    pub heavy_bullet_texture: Option<Texture2D>,
    pub boss_bullet_texture: Option<Texture2D>,
    pub health_pack_texture: Option<Texture2D>,
}

// ==================== 游戏初始化 ====================

impl Game {
    pub fn new(pool: mysql::Pool) -> Self {
        let mut game = Self {
            state: GameState::MainMenu,
            player: Player::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            items: Vec::new(),
            start_time: Instant::now(),
            last_spawn_time: Instant::now(),
            coins: 0,
            wins: 0,
            available_upgrades: Vec::new(),
            screen_width: 800.0,
            screen_height: 600.0,
            rng: ::rand::thread_rng(),
            user: User::new(),
            pool,  // 使用传入的pool
            input_text: String::new(),
            input_mode: InputMode::None,
            current_session_coins: 0,
            current_session_exp: 0,
            enemies_defeated_this_session: 0,
            total_damage_dealt: 0,
            game_result: None,
            rogue_selection_timer: 0.0,
            current_rogue_options: Vec::new(),
            rogue_auto_selected: false,
            rogue_auto_selected_timer: 0.0,
            player_texture: None,
            scout_texture: None,
            heavy_texture: None,
            carrier_texture: None,
            boss_texture: None,
            machinegun_bullet_texture: None,
            laser_bullet_texture: None,
            shotgun_bullet_texture: None,
            heavy_bullet_texture: None,
            boss_bullet_texture: None,
            health_pack_texture: None,
        };
        
        game.init_rogue_upgrades();
        game
    }
    
    fn init_rogue_upgrades(&mut self) {
        self.available_upgrades = vec![
            RogueUpgrade::new(0, "Life-Enhancing", "HP+3", "Grants +3 Maximum HP and restores health over time.", "♥", UpgradeRarity::Common, None),
            RogueUpgrade::new(1, "Firepower Increase", "ATK+2", "Increases base weapon damage by +2.", "⚔", UpgradeRarity::Common, None),
            RogueUpgrade::new(2, "Precision Shooting", "CRIT+10%", "Boosts critical strike chance by 10% and enhances accuracy.", "◉", UpgradeRarity::Rare, None),
            RogueUpgrade::new(3, "Mortal Blow", "CRITDMG+20%", "Increases critical strike damage by 20%, making each crit deadlier.", "✦", UpgradeRarity::Epic, None),
            RogueUpgrade::new(4, "Multi-shot", "BULLET+1", "Fires +1 additional bullet, stacking up to 5 times.", "※", UpgradeRarity::Common, Some(5)),
            RogueUpgrade::new(5, "Exploding Warhead", "EXPLOSION+30%", "Bullets deal 30% splash damage to nearby enemies.", "💥", UpgradeRarity::Rare, None),
            RogueUpgrade::new(6, "Incendiary Ammunition", "BURNING+2", "Bullets ignite enemies, dealing +2 burning damage over time.", "♨", UpgradeRarity::Common, None),
            RogueUpgrade::new(7, "Overclocking Engine", "SPEED+30%", "Increases attack speed and projectile speed by 30%.", "⚡", UpgradeRarity::Rare, None),
            RogueUpgrade::new(8, "Vibranium Armor", "DEF+3", "Reduces incoming damage by 3.", "◊", UpgradeRarity::Epic, None),
            RogueUpgrade::new(9, "Armor Piercing Shell", "PIERCE+1", "Bullets pierce through 1 additional enemy.", "►", UpgradeRarity::Rare, None),
            RogueUpgrade::new(10, "Bouncing Technology", "BOUNCE+1", "Bullets bounce to 1 additional target.", "◈", UpgradeRarity::Epic, None),
        ];
    }
}

// ==================== 游戏逻辑 ====================

impl Game {
    pub fn start_battle(&mut self) {
        self.state = GameState::Battle;
        self.start_time = Instant::now();
        
        // 保存当前武器
        let selected_weapon = self.player.weapon.clone();
        
        // 重新创建玩家但保留武器选择
        self.player = Player::new();
        self.player.weapon = selected_weapon;
        
        // 清空游戏状态
        self.enemies.clear();
        self.bullets.clear();
        self.items.clear();
        
        // 重置本局统计数据
        self.current_session_coins = 0;
        self.current_session_exp = 0;
        self.enemies_defeated_this_session = 0;
        self.total_damage_dealt = 0;
        self.game_result = None;
        
        // 重新初始化肉鸽升级
        self.init_rogue_upgrades();
    }
    
    pub fn update(&mut self, dt: f32) {
        match self.state {
            GameState::Battle => self.update_battle(dt),
            GameState::RogueSelection => self.update_rogue_selection(),
            _ => {}
        }
    }
    
    fn update_battle(&mut self, dt: f32) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        
        // 生成敌人
        self.spawn_enemies(elapsed);
        
        // 更新玩家
        self.update_player(dt);
        
        // 更新敌人
        self.update_enemies(dt, elapsed);
        
        // 更新子弹
        self.update_bullets(dt);
        
        // 更新道具
        self.update_items(dt);
        
        // 碰撞检测
        self.check_collisions();
        
        // 道具碰撞检测
        self.check_item_collisions();
        
        // 检查游戏结束条件
        self.check_game_over(elapsed);
        
        // 检查等级提升
        if self.player.experience >= self.player.experience_needed {
            self.trigger_rogue_selection();
        }
    }
    
    fn spawn_enemies(&mut self, elapsed: f32) {
        if elapsed >= 180.0 { // 3分钟后Boss出现
            if self.enemies.iter().any(|e| e.enemy_type == EnemyType::Boss) {
                return;
            }
            
            let boss_pos = Vec2::new(self.screen_width / 2.0, 50.0);
            self.enemies.push(Enemy::new(EnemyType::Boss, boss_pos));
            return;
        }
        
        if self.last_spawn_time.elapsed().as_secs_f32() < 1.0 {
            return;
        }
        
        let center_margin = self.screen_width * 0.2;
        let center_left = center_margin;
        let center_right = self.screen_width - center_margin;
        let center_top = 50.0;
        let center_bottom = 200.0;
        
        // 生成航母
        if elapsed >= 40.0 && ((elapsed as i32) % 60 == 0 || (elapsed >= 40.0 && elapsed < 45.0 && self.enemies.iter().all(|e| e.enemy_type != EnemyType::Carrier))) {
            let carrier_pos = Vec2::new(
                self.rng.gen_range(center_left..center_right),
                self.rng.gen_range(center_top..center_bottom)
            );
            self.enemies.push(Enemy::new(EnemyType::Carrier, carrier_pos));
        }
        
        // 生成普通敌人
        let scout_count = 3 + (elapsed / 60.0) as i32;
        let heavy_count = if elapsed < 20.0 { 0 } else { 1 + ((elapsed - 20.0) / 30.0) as i32 };
        
        if (elapsed as i32) % 5 == 0 {
            for _ in 0..scout_count {
                let scout_pos = Vec2::new(
                    self.rng.gen_range(center_left..center_right),
                    self.rng.gen_range(center_top..center_bottom)
                );
                self.enemies.push(Enemy::new(EnemyType::Scout, scout_pos));
            }
        }
        
        if (elapsed as i32) % 10 == 0 && elapsed >= 20.0 {
            for _ in 0..heavy_count {
                let heavy_pos = Vec2::new(
                    self.rng.gen_range(center_left..center_right),
                    self.rng.gen_range(center_top..center_bottom)
                );
                self.enemies.push(Enemy::new(EnemyType::Heavy, heavy_pos));
            }
        }
        
        self.last_spawn_time = Instant::now();
    }
    
    fn update_player(&mut self, _dt: f32) {
        // 自动射击
        if self.player.can_shoot() {
            self.player_shoot();
            self.player.last_shot_time = Instant::now();
        }
    }
    
    fn player_shoot(&mut self) {
        let bullet_count = self.player.get_total_bullet_count() as usize;
        let attack_power = self.player.get_total_attack_power();
        
        // 创建带有玩家属性的子弹
        let create_bullet = |pos: Vec2, vel: Vec2, damage: i32, player: &Player, rng: &mut ThreadRng, bullet_type: BulletType| {
            let mut bullet = Bullet::new(pos, vel, damage, true, bullet_type);
            bullet.piercing_count = match player.weapon.weapon_type {
                WeaponType::Laser => 9999,
                _ => player.piercing,
            };
            bullet.ricochet_count = player.ricochet;
            bullet.burning_damage = player.burning_damage;
            bullet.explosion_damage = player.explosion_damage;
            
            if rng.gen_range(0.0..1.0) < player.crit_rate {
                bullet.damage = (bullet.damage as f32 * player.crit_damage) as i32;
                bullet.is_crit = true;
            }
            bullet
        };
        
        match self.player.weapon.weapon_type {
            WeaponType::MachineGun => {
                for i in 0..bullet_count {
                    let offset_x = if i % 2 == 0 { -15.0 } else { 15.0 };
                    let bullet_pos = Vec2::new(self.player.position.x + offset_x, self.player.position.y - 10.0);
                    let bullet_vel = Vec2::new(0.0, -self.player.weapon.bullet_speed * (1.0 + self.player.bullet_speed_bonus));
                    
                    let bullet = create_bullet(bullet_pos, bullet_vel, attack_power, &self.player, &mut self.rng, BulletType::PlayerMachineGun);
                    self.bullets.push(bullet);
                }
            },
            WeaponType::Shotgun => {
                let total_angle = match bullet_count {
                    1 => 0.0, 2 => 30.0, 3 => 45.0, 4 => 60.0, _ => 60.0,
                };
                
                let angle_step = if bullet_count <= 1 { 0.0 } else { total_angle / (bullet_count - 1) as f32 };
                let start_angle = -total_angle / 2.0;
                
                for i in 0..bullet_count {
                    let angle = if bullet_count <= 1 { 0.0 } else { start_angle + angle_step * i as f32 };
                    let rad = angle.to_radians();
                    let bullet_pos = Vec2::new(self.player.position.x, self.player.position.y - 10.0);
                    let speed = self.player.weapon.bullet_speed * (1.0 + self.player.bullet_speed_bonus);
                    let bullet_vel = Vec2::new(rad.sin() * speed, -rad.cos() * speed);
                    
                    let bullet = create_bullet(bullet_pos, bullet_vel, attack_power, &self.player, &mut self.rng, BulletType::PlayerShotgun);
                    self.bullets.push(bullet);
                }
            },
            WeaponType::Laser => {
                for i in 0..bullet_count {
                    let offset_x = if bullet_count > 1 { (i as f32 - (bullet_count - 1) as f32 / 2.0) * 5.0 } else { 0.0 };
                    let bullet_pos = Vec2::new(self.player.position.x + offset_x, self.player.position.y - 10.0);
                    let bullet_vel = Vec2::new(0.0, -8.0 * (1.0 + self.player.bullet_speed_bonus));
                    
                    let bullet = create_bullet(bullet_pos, bullet_vel, attack_power, &self.player, &mut self.rng, BulletType::PlayerLaser);
                    self.bullets.push(bullet);
                }
            }
        }
    }
    
    fn update_enemies(&mut self, dt: f32, elapsed: f32) {
        let mut new_enemies = Vec::new();
        let mut new_bullets = Vec::new();
        let screen_width = self.screen_width;
        let screen_height = self.screen_height;
        let player_position = self.player.position;
        
        for enemy in &mut self.enemies {
            enemy.movement_timer += dt;
            
            match enemy.enemy_type {
                EnemyType::Scout => {
                    enemy.position.y += enemy.velocity.y * dt * 100.0;
                },
                EnemyType::Heavy => {
                    if !enemy.has_reached_zone {
                        enemy.position.y += enemy.velocity.y * dt * 100.0;
                        if enemy.position.y >= enemy.target_position.y {
                            enemy.has_reached_zone = true;
                            enemy.velocity = Vec2::new(0.0, 0.0);
                            enemy.movement_timer = 0.0;
                        }
                    } else {
                        Self::update_heavy_movement(enemy, dt, screen_width, player_position);
                    }
                    
                    if enemy.last_shot_time.elapsed().as_secs_f32() >= 1.0 {
                        let attack_pattern = (enemy.movement_timer as i32) % 4;
                        
                        match attack_pattern {
                            0 => {
                                if elapsed >= 90.0 {
                                    let target_dir = Vec2::new(
                                        player_position.x - enemy.position.x,
                                        player_position.y - enemy.position.y
                                    ).normalize();
                                    
                                    for i in 0..3 {
                                        let spread_angle = (-10.0 + i as f32 * 10.0).to_radians();
                                        let bullet_vel = Vec2::new(
                                            target_dir.x * 2.5 + spread_angle.sin() * 0.5,
                                            target_dir.y * 2.5 + spread_angle.cos() * 0.5
                                        );
                                        let bullet_pos = Vec2::new(enemy.position.x, enemy.position.y + 20.0);
                                        new_bullets.push(Bullet::new(bullet_pos, bullet_vel, enemy.bullet_damage + 1, false, BulletType::EnemyHeavy));
                                    }
                                }
                            },
                            1 => {
                                if elapsed >= 90.0 {
                                    for i in 0..5 {
                                        let angle = (-30.0 + i as f32 * 15.0).to_radians();
                                        let bullet_pos = Vec2::new(enemy.position.x, enemy.position.y + 20.0);
                                        let bullet_vel = Vec2::new(angle.sin() * 2.0, angle.cos() * 2.0 + 1.0);
                                        new_bullets.push(Bullet::new(bullet_pos, bullet_vel, enemy.bullet_damage, false, BulletType::EnemyGeneric));
                                    }
                                }
                            },
                            2 => {
                                let bullet_pos1 = Vec2::new(enemy.position.x - 10.0, enemy.position.y + 20.0);
                                let bullet_pos2 = Vec2::new(enemy.position.x + 10.0, enemy.position.y + 20.0);
                                let bullet_vel = Vec2::new(0.0, 3.0);
                                new_bullets.push(Bullet::new(bullet_pos1, bullet_vel, enemy.bullet_damage, false, BulletType::EnemyGeneric));
                                new_bullets.push(Bullet::new(bullet_pos2, bullet_vel, enemy.bullet_damage, false, BulletType::EnemyGeneric));
                            },
                            3 => {
                                let predict_pos = Vec2::new(
                                    player_position.x,
                                    player_position.y + 50.0
                                );
                                let target_dir = Vec2::new(
                                    predict_pos.x - enemy.position.x,
                                    predict_pos.y - enemy.position.y
                                ).normalize();
                                
                                let bullet_pos = Vec2::new(enemy.position.x, enemy.position.y + 20.0);
                                let bullet_vel = Vec2::new(target_dir.x * 3.0, target_dir.y * 3.0);
                                new_bullets.push(Bullet::new(bullet_pos, bullet_vel, enemy.bullet_damage + 2, false, BulletType::EnemyBoss));
                            },
                            _ => {}
                        }
                        enemy.last_shot_time = Instant::now();
                    }
                },
                EnemyType::Carrier => {
                    enemy.position.y += enemy.velocity.y * dt * 100.0;
                    
                    if enemy.last_shot_time.elapsed().as_secs_f32() >= 5.0 {
                        let scout_pos = Vec2::new(enemy.position.x, enemy.position.y + 30.0);
                        new_enemies.push(Enemy::new(EnemyType::Scout, scout_pos));
                        enemy.last_shot_time = Instant::now();
                    }
                },
                EnemyType::Boss => {
                    if !enemy.has_reached_zone {
                        enemy.position.y += enemy.velocity.y * dt * 100.0;
                        if enemy.position.y >= enemy.target_position.y {
                            enemy.has_reached_zone = true;
                            enemy.velocity = Vec2::new(1.0, 0.0);
                            enemy.movement_timer = 0.0;
                        }
                    } else {
                        Self::update_boss_movement(enemy, dt, screen_width);
                    }
                    
                    let boss_bullets = Self::update_boss_and_get_bullets(enemy, elapsed);
                    new_bullets.extend(boss_bullets);
                }
            }
        }
        
        self.enemies.extend(new_enemies);
        self.bullets.extend(new_bullets);
        
        self.enemies.retain(|enemy| {
            match enemy.enemy_type {
                EnemyType::Heavy | EnemyType::Boss => {
                    if enemy.has_reached_zone {
                        enemy.health > 0
                    } else {
                        enemy.position.y < screen_height + 50.0 && 
                        enemy.position.x > -50.0 && 
                        enemy.position.x < screen_width + 50.0
                    }
                },
                _ => {
                    enemy.position.y < screen_height + 50.0 && 
                    enemy.position.x > -50.0 && 
                    enemy.position.x < screen_width + 50.0
                }
            }
        });
    }
    
    fn update_heavy_movement(enemy: &mut Enemy, dt: f32, screen_width: f32, player_position: Vec2) {
        let speed = 50.0;
        let battle_zone_top = 80.0;
        let battle_zone_bottom = 200.0;
        let margin = 50.0;
        let mut rng = ::rand::thread_rng();
        
        match enemy.movement_pattern {
            1 => {
                if enemy.movement_timer >= 3.0 {
                    enemy.velocity.x = -enemy.velocity.x;
                    enemy.movement_timer = 0.0;
                }
                if enemy.velocity.x == 0.0 {
                    enemy.velocity.x = if rng.gen_bool(0.5) { speed } else { -speed };
                }
                enemy.position.x += enemy.velocity.x * dt;
                
                if enemy.position.x <= margin {
                    enemy.position.x = margin;
                    enemy.velocity.x = speed;
                }
                if enemy.position.x >= screen_width - margin {
                    enemy.position.x = screen_width - margin;
                    enemy.velocity.x = -speed;
                }
            },
            2 => {
                let radius = 60.0;
                let angular_speed = 1.0;
                let center_x = enemy.target_position.x;
                let center_y = (battle_zone_top + battle_zone_bottom) / 2.0;
                
                let angle = enemy.movement_timer * angular_speed;
                enemy.position.x = center_x + radius * angle.cos();
                enemy.position.y = center_y + radius * 0.5 * angle.sin();
            },
            3 => {
                let zigzag_speed = 40.0;
                if enemy.movement_timer >= 2.0 {
                    enemy.velocity = Vec2::new(
                        if rng.gen_bool(0.5) { zigzag_speed } else { -zigzag_speed },
                        if rng.gen_bool(0.5) { zigzag_speed * 0.5 } else { -zigzag_speed * 0.5 }
                    );
                    enemy.movement_timer = 0.0;
                }
                
                enemy.position.x += enemy.velocity.x * dt;
                enemy.position.y += enemy.velocity.y * dt;
                
                enemy.position.x = enemy.position.x.clamp(margin, screen_width - margin);
                enemy.position.y = enemy.position.y.clamp(battle_zone_top, battle_zone_bottom);
            },
            4 => {
                let pursuit_speed = 30.0;
                let dx = player_position.x - enemy.position.x;
                if dx.abs() > 20.0 {
                    let direction = if dx > 0.0 { 1.0 } else { -1.0 };
                    enemy.position.x += direction * pursuit_speed * dt;
                }
                
                if enemy.movement_timer >= 4.0 {
                    enemy.target_position.y = rng.gen_range(battle_zone_top..battle_zone_bottom);
                    enemy.movement_timer = 0.0;
                }
                
                let dy = enemy.target_position.y - enemy.position.y;
                if dy.abs() > 5.0 {
                    let direction = if dy > 0.0 { 1.0 } else { -1.0 };
                    enemy.position.y += direction * 20.0 * dt;
                }
                
                enemy.position.x = enemy.position.x.clamp(margin, screen_width - margin);
            },
            _ => {}
        }
    }
    
    fn update_boss_movement(enemy: &mut Enemy, dt: f32, screen_width: f32) {
        let speed = 80.0;
        let margin = 80.0;
        
        enemy.position.x += enemy.velocity.x * dt * speed;
        
        if enemy.position.x <= margin {
            enemy.position.x = margin;
            enemy.velocity.x = 1.0;
        }
        if enemy.position.x >= screen_width - margin {
            enemy.position.x = screen_width - margin;
            enemy.velocity.x = -1.0;
        }
        
        let float_amplitude = 10.0;
        let float_frequency = 2.0;
        let base_y = enemy.target_position.y;
        enemy.position.y = base_y + float_amplitude * (enemy.movement_timer * float_frequency).sin();
    }
    
    fn update_boss_and_get_bullets(boss: &mut Enemy, _elapsed: f32) -> Vec<Bullet> {
        let mut new_bullets = Vec::new();
        let boss_time = boss.spawn_time.elapsed().as_secs_f32();
        
        if boss.special_state == 1 {
            if boss.last_shot_time.elapsed().as_secs_f32() >= 3.0 {
                let attack_cycle = (boss_time as i32) % 6;
                
                match attack_cycle {
                    0 => {
                        for i in 0..24 {
                            let angle = (i as f32 * 15.0).to_radians();
                            let bullet_pos = Vec2::new(boss.position.x, boss.position.y + 50.0);
                            let bullet_vel = Vec2::new(angle.cos() * 1.5, angle.sin() * 1.5);
                            new_bullets.push(Bullet::new(bullet_pos, bullet_vel, boss.bullet_damage, false, BulletType::EnemyBoss));
                        }
                    },
                    1..=5 => {
                        // 其他攻击模式...
                        for i in 0..12 {
                            let angle = (i as f32 * 30.0).to_radians();
                            let bullet_pos = Vec2::new(boss.position.x, boss.position.y + 50.0);
                            let bullet_vel = Vec2::new(angle.cos() * 2.0, angle.sin() * 2.0);
                            new_bullets.push(Bullet::new(bullet_pos, bullet_vel, boss.bullet_damage, false, BulletType::EnemyBoss));
                        }
                    },
                    _ => {}
                }
                boss.last_shot_time = Instant::now();
            }
        } else if boss.special_state == 2 {
            if boss.is_invincible && boss.spawn_time.elapsed().as_secs_f32() >= 5.0 {
                boss.is_invincible = false;
            }
            
            if boss.last_shot_time.elapsed().as_secs_f32() >= 2.0 {
                for i in 0..32 {
                    let angle = (i as f32 * 11.25).to_radians();
                    let bullet_pos = Vec2::new(boss.position.x, boss.position.y + 50.0);
                    let bullet_vel = Vec2::new(angle.cos() * 2.5, angle.sin() * 2.5);
                    new_bullets.push(Bullet::new(bullet_pos, bullet_vel, 15, false, BulletType::EnemyBoss));
                }
                boss.last_shot_time = Instant::now();
            }
        }
        
        new_bullets
    }
    
    fn update_bullets(&mut self, dt: f32) {
        for bullet in &mut self.bullets {
            bullet.position.x += bullet.velocity.x * dt * 100.0;
            bullet.position.y += bullet.velocity.y * dt * 100.0;
            
            if bullet.ricochet_count > 0 {
                let mut bounced = false;
                if bullet.position.x <= 0.0 || bullet.position.x >= self.screen_width {
                    bullet.velocity.x = -bullet.velocity.x;
                    bullet.ricochet_count -= 1;
                    bounced = true;
                }
                if bullet.position.y <= 0.0 || bullet.position.y >= self.screen_height {
                    bullet.velocity.y = -bullet.velocity.y;
                    bullet.ricochet_count -= 1;
                    bounced = true;
                }
                if bounced {
                    bullet.position.x = bullet.position.x.clamp(0.0, self.screen_width);
                    bullet.position.y = bullet.position.y.clamp(0.0, self.screen_height);
                    bullet.hit_enemies.clear();
                }
            }
        }
        
        self.bullets.retain(|bullet| {
            if bullet.ricochet_count > 0 {
                true
            } else {
                bullet.position.y > -50.0 && bullet.position.y < self.screen_height + 50.0 &&
                bullet.position.x > -50.0 && bullet.position.x < self.screen_width + 50.0
            }
        });
    }
    
    fn update_items(&mut self, dt: f32) {
        for item in &mut self.items {
            item.position.y += item.velocity.y * dt * 50.0;
        }
        
        self.items.retain(|item| item.position.y < self.screen_height + 50.0);
    }
    
    fn check_item_collisions(&mut self) {
        let mut items_to_remove = Vec::new();
        
        for (item_idx, item) in self.items.iter().enumerate() {
            let distance = item.position.distance(&self.player.position);
            if distance < 25.0 {
                match item.item_type {
                    ItemType::HealthPack => {
                        self.player.health = (self.player.health + item.value).min(self.player.max_health);
                    }
                }
                items_to_remove.push(item_idx);
            }
        }
        
        items_to_remove.sort_unstable();
        items_to_remove.reverse();
        for idx in items_to_remove {
            if idx < self.items.len() {
                self.items.remove(idx);
            }
        }
    }
    
    fn check_collisions(&mut self) {
        let mut bullets_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();
        let mut explosion_damages = Vec::new();
        let mut enemy_bullet_hits = Vec::new();
        let mut bullet_piercing_updates = Vec::new();
        let mut bullet_hit_updates = Vec::new();
        
        // 子弹与敌人碰撞
        for (bullet_idx, bullet) in self.bullets.iter().enumerate() {
            if !bullet.is_player_bullet {
                continue;
            }
            
            let mut should_remove_bullet = false;
            let mut new_hit_enemies = bullet.hit_enemies.clone();
            
            for (enemy_idx, enemy) in self.enemies.iter().enumerate() {
                if enemy.health <= 0 || bullet.hit_enemies.contains(&enemy_idx) {
                    continue;
                }
                
                let distance = bullet.position.distance(&enemy.position);
                if distance < 30.0 {
                    new_hit_enemies.push(enemy_idx);
                    
                    let mut damage = bullet.damage;
                    if bullet.burning_damage > 0 {
                        damage += bullet.burning_damage;
                    }
                    
                    enemy_bullet_hits.push((enemy_idx, damage));
                    self.total_damage_dealt += damage;
                    
                    if bullet.explosion_damage > 0.0 {
                        let explosion_dmg = (damage as f32 * bullet.explosion_damage) as i32;
                        explosion_damages.push((enemy.position, explosion_dmg));
                    }
                    
                    if bullet.piercing_count != 9999 && bullet.piercing_count > 0 {
                        bullet_piercing_updates.push((bullet_idx, bullet.piercing_count - 1));
                        if bullet.piercing_count - 1 <= 0 {
                            should_remove_bullet = true;
                        }
                    } else if bullet.piercing_count == 0 {
                        should_remove_bullet = true;
                    }
                    
                    if bullet.piercing_count == 0 {
                        break;
                    }
                }
            }
            
            if new_hit_enemies.len() > bullet.hit_enemies.len() {
                bullet_hit_updates.push((bullet_idx, new_hit_enemies));
            }
            
            if should_remove_bullet {
                bullets_to_remove.push(bullet_idx);
            }
        }
        
        // 更新子弹数据
        for (bullet_idx, new_hit_list) in bullet_hit_updates {
            if let Some(bullet) = self.bullets.get_mut(bullet_idx) {
                bullet.hit_enemies = new_hit_list;
            }
        }
        
        for (bullet_idx, new_piercing) in bullet_piercing_updates {
            if let Some(bullet) = self.bullets.get_mut(bullet_idx) {
                bullet.piercing_count = new_piercing;
            }
        }
        
        // 应用伤害
        for (enemy_idx, damage) in enemy_bullet_hits {
            if let Some(enemy) = self.enemies.get_mut(enemy_idx) {
                enemy.take_damage(damage);
                
                if enemy.health <= 0 {
                    let coins = enemy.get_drop_gold();
                    let exp = enemy.get_drop_exp();
                    
                    self.current_session_coins += coins;
                    self.current_session_exp += exp;
                    self.enemies_defeated_this_session += 1;
                    
                    self.coins += coins;
                    self.player.add_experience(exp);
                    
                    // 重甲舰掉落道具
                    if enemy.enemy_type == EnemyType::Heavy {
                        if self.rng.gen_range(0.0..1.0) < 0.4 {
                            let health_pack = Item::new(
                                ItemType::HealthPack, 
                                enemy.position.clone(), 
                                30
                            );
                            self.items.push(health_pack);
                        }
                    }
                    
                    enemies_to_remove.push(enemy_idx);
                }
            }
        }
        
        // 处理爆炸效果
        for (explosion_pos, explosion_dmg) in explosion_damages {
            for (enemy_idx, enemy) in self.enemies.iter_mut().enumerate() {
                if enemy.health > 0 && enemy.position.distance(&explosion_pos) < 50.0 {
                    enemy.take_damage(explosion_dmg);
                    self.total_damage_dealt += explosion_dmg;
                    
                    if enemy.health <= 0 {
                        let coins = enemy.get_drop_gold();
                        let exp = enemy.get_drop_exp();
                        
                        self.current_session_coins += coins;
                        self.current_session_exp += exp;
                        self.enemies_defeated_this_session += 1;
                        
                        self.coins += coins;
                        self.player.add_experience(exp);
                        
                        if enemy.enemy_type == EnemyType::Heavy {
                            if self.rng.gen_range(0.0..1.0) < 0.4 {
                                let health_pack = Item::new(
                                    ItemType::HealthPack, 
                                    enemy.position.clone(), 
                                    30
                                );
                                self.items.push(health_pack);
                            }
                        }
                        
                        enemies_to_remove.push(enemy_idx);
                    }
                }
            }
        }
        
        // 敌人子弹与玩家碰撞
        for (bullet_idx, bullet) in self.bullets.iter().enumerate() {
            if bullet.is_player_bullet {
                continue;
            }
            let distance = bullet.position.distance(&self.player.position);
            if distance < 25.0 {
                self.player.take_damage(bullet.damage);
                bullets_to_remove.push(bullet_idx);
            }
        }
        
        // 敌人与玩家碰撞
        for enemy in &self.enemies {
            let distance = enemy.position.distance(&self.player.position);
            if distance < 30.0 {
                self.player.take_damage(enemy.collision_damage);
            }
        }
        
        // 移除子弹和敌人
        bullets_to_remove.sort_unstable();
        bullets_to_remove.reverse();
        for idx in bullets_to_remove {
            if idx < self.bullets.len() {
                self.bullets.remove(idx);
            }
        }
        
        enemies_to_remove.sort_unstable();
        enemies_to_remove.dedup();
        enemies_to_remove.reverse();
        for idx in enemies_to_remove {
            if idx < self.enemies.len() {
                self.enemies.remove(idx);
                
                for bullet in &mut self.bullets {
                    bullet.hit_enemies.retain(|&enemy_idx| enemy_idx != idx);
                    for hit_idx in &mut bullet.hit_enemies {
                        if *hit_idx > idx {
                            *hit_idx -= 1;
                        }
                    }
                }
            }
        }
    }
    
    fn check_game_over(&mut self, elapsed: f32) {
        if self.player.health <= 0 {
            self.end_game(false);
            return;
        }
        
        if elapsed >= 180.0 {
            let boss_alive = self.enemies.iter().any(|e| e.enemy_type == EnemyType::Boss && e.health > 0);
            let boss_ever_spawned = elapsed >= 180.0;
            
            if boss_ever_spawned && !boss_alive {
                self.end_game(true);
            }
        }
    }
    
    fn end_game(&mut self, victory: bool) {
        let survival_time = self.get_game_time();
        
        let mut game_result = GameResult::new(
            &self.player,
            victory,
            survival_time,
            self.enemies_defeated_this_session,
            self.total_damage_dealt
        );
        
        game_result.coins_earned = self.current_session_coins;
        game_result.experience_gained = self.current_session_exp;
        
        self.game_result = Some(game_result);
        
        if victory {
            self.wins += 1;
        }
        
        self.state = GameState::GameOver;
        self.reset_game_progress();
    }
    
    fn reset_game_progress(&mut self) {
        self.coins = 0;
        
        let current_weapon = self.player.weapon.clone();
        self.player = Player::new();
        self.player.weapon = current_weapon;
        
        self.enemies.clear();
        self.bullets.clear();
        
        self.current_session_coins = 0;
        self.current_session_exp = 0;
        self.enemies_defeated_this_session = 0;
        self.total_damage_dealt = 0;
        
        self.init_rogue_upgrades();
    }
    
    pub fn get_game_result(&self) -> Option<&GameResult> {
        self.game_result.as_ref()
    }
    
    fn trigger_rogue_selection(&mut self) {
        self.current_rogue_options = self.get_random_rogue_options();
        
        self.rogue_selection_timer = 0.0;
        self.rogue_auto_selected = false;
        self.rogue_auto_selected_timer = 0.0;
        
        self.state = GameState::RogueSelection;
    }
    
    fn get_random_rogue_options(&mut self) -> Vec<RogueUpgrade> {
        let mut options = Vec::new();
        let mut available = self.available_upgrades.clone();
        
        for _ in 0..3.min(available.len()) {
            if available.is_empty() {
                break;
            }
            
            let index = self.rng.gen_range(0..available.len());
            let upgrade = available.remove(index);
            options.push(upgrade);
        }
        
        options
    }
    
    fn update_rogue_selection(&mut self) {
        if self.rogue_auto_selected {
            self.rogue_auto_selected_timer += get_frame_time();
            if self.rogue_auto_selected_timer >= 2.0 {
                self.complete_rogue_selection();
            }
        } else {
            self.rogue_selection_timer += get_frame_time();
            
            if self.rogue_selection_timer >= 10.0 {
                self.auto_select_rogue_upgrade();
            }
        }
    }
    
    fn auto_select_rogue_upgrade(&mut self) {
        if !self.current_rogue_options.is_empty() && !self.rogue_auto_selected {
            let random_index = self.rng.gen_range(0..self.current_rogue_options.len());
            let selected_upgrade = self.current_rogue_options[random_index].clone();
            
            self.apply_upgrade_and_complete(selected_upgrade);
            
            self.rogue_auto_selected = true;
            self.rogue_auto_selected_timer = 0.0;
        }
    }
    
    fn apply_upgrade_and_complete(&mut self, upgrade: RogueUpgrade) {
        self.apply_rogue_upgrade(upgrade.id);
        
        self.player.rogue_upgrades.push(upgrade.clone());
        
        if let Some(available_upgrade) = self.available_upgrades.iter_mut().find(|u| u.id == upgrade.id) {
            available_upgrade.current_selections += 1;
            
            if let Some(max) = available_upgrade.max_selections {
                if available_upgrade.current_selections >= max {
                    self.available_upgrades.retain(|u| u.id != upgrade.id);
                }
            }
        }
    }
    
    fn apply_rogue_upgrade(&mut self, upgrade_id: u32) {
        match upgrade_id {
            0 => {
                self.player.max_health += 3;
                self.player.health += 3;
            },
            1 => self.player.attack_power_bonus += 2,
            2 => self.player.crit_rate += 0.1,
            3 => self.player.crit_damage += 0.2,
            4 => self.player.bullet_count_bonus += 1,
            5 => self.player.explosion_damage += 0.3,
            6 => self.player.burning_damage += 2,
            7 => {
                self.player.bullet_speed_bonus += 0.3;
                self.player.weapon.attack_speed *= 1.3;
            },
            8 => self.player.damage_reduction += 3,
            9 => self.player.piercing += 1,
            10 => self.player.ricochet += 1,
            _ => {}
        }
    }
    
    fn complete_rogue_selection(&mut self) {
        self.player.level_up();
        
        self.current_rogue_options.clear();
        
        if self.player.experience >= self.player.experience_needed {
            self.trigger_rogue_selection();
        } else {
            self.state = GameState::Battle;
        }
    }
    
    pub fn select_rogue_upgrade(&mut self, option_index: usize) {
        if self.state != GameState::RogueSelection || self.rogue_auto_selected {
            return;
        }
        
        if option_index < self.current_rogue_options.len() {
            let selected_upgrade = self.current_rogue_options[option_index].clone();
            
            self.apply_upgrade_and_complete(selected_upgrade);
            
            self.complete_rogue_selection();
        }
    }
    
    pub fn move_player(&mut self, dx: f32, dy: f32) {
        let new_x = (self.player.position.x + dx).clamp(25.0, self.screen_width - 25.0);
        let new_y = (self.player.position.y + dy).clamp(25.0, self.screen_height - 25.0);
        self.player.position = Vec2::new(new_x, new_y);
    }
    
    pub fn get_game_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
    
    pub fn select_weapon(&mut self, weapon_type: WeaponType) {
        self.player.weapon = Weapon::new(weapon_type);
        self.start_battle();
    }
    
    pub fn login_attempt(&mut self) -> bool {
        let username = self.user.username.clone();
        let password = self.input_text.clone();
        
        match self.user.login(&self.pool, &username, &password) {
            Ok(true) => {
                self.state = GameState::MainMenu;
                true
            }
            Ok(false) => {
                println!("用户名或密码错误");
                false
            }
            Err(e) => {
                println!("数据库错误: {}", e);
                false
            }
        }
    }


    
    pub fn add_char_to_input(&mut self, ch: char) {
        if self.input_text.len() < 20 {
            self.input_text.push(ch);
        }
    }
    
    pub fn remove_char_from_input(&mut self) {
        self.input_text.pop();
    }
    
    pub fn clear_input(&mut self) {
        self.input_text.clear();
    }
}

// ==================== 配置窗口 ====================

fn window_conf() -> Conf {
    Conf {
        window_title: "Puppy WCP And His Four Dom Masters".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

// ==================== 纹理加载辅助函数 ====================

async fn load_game_texture(path: &str, name: &str) -> Option<Texture2D> {
    println!("Loading {} texture...", name);
    match load_texture(path).await {
        Ok(texture) => {
            println!("{} texture loaded successfully!", name);
            Some(texture)
        },
        Err(e) => {
            println!("Failed to load {} texture: {}", name, e);
            None
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
   // ==================== 数据库初始化 ====================
    let url = "mysql://root:kindi@172.20.26.118:3306/msb";
    let pool = match mysql::Pool::new(url) {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("❗ 数据库连接失败: {}", e);
            eprintln!("💡 请检查：\n1. MySQL服务是否启动\n2. 连接字符串是否正确\n3. 用户名/密码是否有权限");
            return; // 如果数据库连接失败，直接退出程序
        }
    };
    // 初始化游戏对象
    let mut game = Game::new(pool);
    
    // 批量加载所有纹理
    println!("=== Loading Game Textures ===");
    
    // 加载玩家和敌人纹理
    game.player_texture = load_game_texture("resources/fighter.png", "Player").await;
    game.scout_texture = load_game_texture("resources/Scout.png", "Scout").await;
    game.heavy_texture = load_game_texture("resources/Heavy.png", "Heavy").await;
    game.carrier_texture = load_game_texture("resources/Carrier.png", "Carrier").await;
    game.boss_texture = load_game_texture("resources/Boss.png", "Boss").await;
    
    // 加载子弹纹理
    game.machinegun_bullet_texture = load_game_texture("resources/MachineGun.png", "MachineGun Bullet").await;
    game.laser_bullet_texture = load_game_texture("resources/Laser.png", "Laser").await;
    game.shotgun_bullet_texture = load_game_texture("resources/Shotgun.png", "Shotgun Bullet").await;
    game.heavy_bullet_texture = load_game_texture("resources/Heavybullet-1.png", "Heavy Bullet").await;
    game.boss_bullet_texture = load_game_texture("resources/Bossbullet.png", "Boss Bullet").await;
    
    // 加载道具纹理
    game.health_pack_texture = load_game_texture("resources/Health .png", "Health Pack").await;
    
    println!("=== Texture Loading Complete ===");
    
    let mut last_time = get_time();
    
    loop {
        // 计算帧时间
        let current_time = get_time();
        let dt = (current_time - last_time) as f32;
        last_time = current_time;
        
        // 处理输入
        handle_input_macroquad(&mut game);
        
        // 更新游戏逻辑
        game.update(dt);
        
        // 渲染
        clear_background(BLACK);
        render_game(&game);
        
        // 显示UI
        render_ui(&game);
        
        next_frame().await
    }
}

// ==================== 处理输入 ====================

fn handle_input_macroquad(game: &mut Game) {
    match game.state {
        GameState::MainMenu => {
            if is_key_pressed(KeyCode::Key1) {
                game.state = GameState::WeaponSelect;
            } else if is_key_pressed(KeyCode::Key2) {
                game.state = GameState::Login;
                game.input_mode = InputMode::Username;
                game.clear_input();
            }
        },
        GameState::WeaponSelect => {
            if is_key_pressed(KeyCode::Key1) {
                game.select_weapon(WeaponType::MachineGun);
            } else if is_key_pressed(KeyCode::Key2) {
                game.select_weapon(WeaponType::Laser);
            } else if is_key_pressed(KeyCode::Key3) {
                game.select_weapon(WeaponType::Shotgun);
            } else if is_key_pressed(KeyCode::Escape) {
                game.state = GameState::MainMenu;
            }
        },
        GameState::Login => {
            handle_login_input(game);
        },
        GameState::Battle => {
            let speed = 300.0;
            let dt = get_frame_time();
            
            // WASD移动
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                game.move_player(0.0, -speed * dt);
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                game.move_player(0.0, speed * dt);
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                game.move_player(-speed * dt, 0.0);
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                game.move_player(speed * dt, 0.0);
            }
            
            // ESC返回主菜单
            if is_key_pressed(KeyCode::Escape) {
                game.state = GameState::MainMenu;
            }
        },
        GameState::RogueSelection => {
            // 数字键选择肉鸽升级 - 更新为使用选项索引
            if is_key_pressed(KeyCode::Key1) {
                game.select_rogue_upgrade(0); // 选择第一个选项
            } else if is_key_pressed(KeyCode::Key2) {
                game.select_rogue_upgrade(1); // 选择第二个选项
            } else if is_key_pressed(KeyCode::Key3) {
                game.select_rogue_upgrade(2); // 选择第三个选项
            }
        },
        GameState::GameOver => {
            // R键重新开始
            if is_key_pressed(KeyCode::R) {
                game.state = GameState::WeaponSelect;
            } else if is_key_pressed(KeyCode::Escape) {
                game.state = GameState::MainMenu;
            }
        },
    }
}

fn handle_login_input(game: &mut Game) {
    // 处理字符输入
    if let Some(character) = get_char_pressed() {
        // 只允许输入字母、数字和常见符号
        if character.is_ascii_alphanumeric() 
            || character == '_' 
            || character == '@' 
            || character == '.' 
            || character == '-' 
        {
            if game.input_text.len() < 20 {  // 限制最大长度
                game.add_char_to_input(character);
            } else {
                println!("⚠️ 输入已达最大长度(20字符)");
            }
        }
    }

    // 处理退格键
    if is_key_pressed(KeyCode::Backspace) {
        game.remove_char_from_input();
    }

    // 处理回车键（提交登录）
    if is_key_pressed(KeyCode::Enter) {
        match game.input_mode {
            InputMode::Username => {
                if !game.input_text.is_empty() {
                    game.user.username = game.input_text.clone();
                    game.clear_input();
                    game.input_mode = InputMode::Password;
                    println!("↪️ 请输入密码");
                }
            }
            InputMode::Password => {
                if !game.input_text.is_empty() {
                    println!("⏳ 正在验证登录信息...");
                    
                    if game.login_attempt() {  // 使用无参数版本
                        println!("✅ 登录成功！欢迎, {}", game.user.username);
                        game.clear_input();
                        game.input_mode = InputMode::None;
                    } else {
                        println!("❌ 登录失败，请重试");
                        game.clear_input();
                        game.input_mode = InputMode::Username;
                        game.user.username.clear();
                    }
                }
            }
            _ => {}
        }
    }

    // 处理ESC键返回
    if is_key_pressed(KeyCode::Escape) {
        println!("⎋ 返回主菜单");
        game.state = GameState::MainMenu;
        game.input_mode = InputMode::None;
        game.clear_input();
    }

    // 调试用：显示当前输入状态
    if is_key_pressed(KeyCode::F1) {
        println!("🔍 调试信息：");
        println!("模式: {:?}", game.input_mode);
        println!("用户名: {}", game.user.username);
        println!("输入内容: {}", game.input_text);
    }
}

// ==================== 渲染游戏对象 ====================

fn render_game(game: &Game) {
    // 绘制玩家 - 优先使用图片，否则使用圆形
    if let Some(texture) = &game.player_texture {
        // 使用飞机图片绘制玩家
        let texture_size = 40.0; // 图片显示大小
        let draw_x = game.player.position.x - texture_size / 2.0;
        let draw_y = game.player.position.y - texture_size / 2.0;
        
        // 根据无敌状态调整颜色
        let tint_color = if game.player.last_damage_time.elapsed().as_secs_f32() < game.player.invincibility_duration {
            // 无敌时间内闪烁效果
            if (get_time() * 10.0) as i32 % 2 == 0 {
                Color::new(0.0, 0.0, 1.0, 0.5) // 半透明蓝色
            } else {
                WHITE // 正常显示
            }
        } else {
            WHITE // 正常显示
        };
        
        // 使用draw_texture_ex来支持旋转和缩放
        draw_texture_ex(
            texture,
            draw_x,
            draw_y,
            tint_color,
            DrawTextureParams {
                dest_size: Some(macroquad::math::Vec2::new(texture_size, texture_size)),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
                source: None,
            }
        );
    } else {
        // 回退到原来的圆形绘制
        let player_color = if game.player.last_damage_time.elapsed().as_secs_f32() < game.player.invincibility_duration {
            // 无敌时间内闪烁效果
            if (get_time() * 10.0) as i32 % 2 == 0 {
                Color::new(0.0, 0.0, 1.0, 0.5) // 半透明蓝色
            } else {
                BLUE
            }
        } else {
            BLUE
        };
        
        draw_circle(
            game.player.position.x,
            game.player.position.y,
            15.0,
            player_color
        );
    }
    
    // 绘制玩家血条
    let player_health_ratio = game.player.health as f32 / game.player.max_health as f32;
    let health_bar_width = 30.0;
    let health_bar_height = 4.0;
    let health_bar_x = game.player.position.x - health_bar_width / 2.0;
    let health_bar_y = game.player.position.y - 30.0; // 稍微向上移动血条位置，给图片留空间
    
    // 血条背景
    draw_rectangle(health_bar_x, health_bar_y, health_bar_width, health_bar_height, DARKGRAY);
    
    // 血条
    let health_color = if player_health_ratio > 0.6 {
        GREEN
    } else if player_health_ratio > 0.3 {
        YELLOW
    } else {
        RED
    };
    draw_rectangle(health_bar_x, health_bar_y, health_bar_width * player_health_ratio, health_bar_height, health_color);
    
    // 绘制敌人
    for enemy in &game.enemies {
        // 根据敌人类型选择纹理和大小
        let (texture_opt, size) = match enemy.enemy_type {
            EnemyType::Scout => (&game.scout_texture, 16.0),      // 侦察机较小
            EnemyType::Heavy => (&game.heavy_texture, 24.0),      // 重甲舰中等
            EnemyType::Carrier => (&game.carrier_texture, 40.0),  // 航母大型
            EnemyType::Boss => (&game.boss_texture, 60.0),        // Boss超大型
        };
        
        // 如果有纹理，使用纹理绘制；否则回退到圆形
        if let Some(texture) = texture_opt {
            let draw_x = enemy.position.x - size / 2.0;
            let draw_y = enemy.position.y - size / 2.0;
            
            // 根据敌人状态调整颜色（无敌状态时变红）
            let tint_color = if enemy.is_invincible {
                Color::new(1.0, 0.5, 0.5, 0.8) // 半透明红色
            } else {
                WHITE // 正常显示
            };
            
            draw_texture_ex(
                texture,
                draw_x,
                draw_y,
                tint_color,
                DrawTextureParams {
                    dest_size: Some(macroquad::math::Vec2::new(size, size)),
                    rotation: 0.0,
                    flip_x: false,
                    flip_y: false,
                    pivot: None,
                    source: None,
                }
            );
        } else {
            // 回退到原来的圆形绘制
            let color = match enemy.enemy_type {
                EnemyType::Scout => RED,
                EnemyType::Heavy => Color::new(0.5, 0.0, 0.0, 1.0),
                EnemyType::Carrier => PURPLE,
                EnemyType::Boss => MAROON,
            };
            
            let circle_size = match enemy.enemy_type {
                EnemyType::Scout => 8.0,
                EnemyType::Heavy => 12.0,
                EnemyType::Carrier => 20.0,
                EnemyType::Boss => 30.0,
            };
            
            draw_circle(enemy.position.x, enemy.position.y, circle_size, color);
        }
        
        // 绘制血条
        if enemy.max_health > 0 {
            let health_ratio = enemy.health as f32 / enemy.max_health as f32;
            let bar_width = size;
            let bar_height = 4.0;
            let bar_x = enemy.position.x - bar_width / 2.0;
            let bar_y = enemy.position.y - size / 2.0 - 8.0;
            
            draw_rectangle(bar_x, bar_y, bar_width, bar_height, DARKGRAY);
            draw_rectangle(bar_x, bar_y, bar_width * health_ratio, bar_height, GREEN);
        }
    }
    
    // 绘制子弹
    for bullet in &game.bullets {
        if bullet.is_player_bullet {
            // 玩家子弹 - 根据子弹类型选择纹理
            let (texture_opt, size) = match bullet.bullet_type {
                BulletType::PlayerMachineGun => (&game.machinegun_bullet_texture, 8.0),
                BulletType::PlayerLaser => (&game.laser_bullet_texture, 12.0),
                BulletType::PlayerShotgun => (&game.shotgun_bullet_texture, 6.0),
                _ => (&None, 3.0), // 默认情况 - 修复类型匹配
            };
            
            if let Some(texture) = texture_opt {
                // 使用纹理绘制
                let draw_x = bullet.position.x - size / 2.0;
                let draw_y = bullet.position.y - size / 2.0;
                
                // 暴击子弹特殊颜色
                let tint_color = if bullet.is_crit { 
                    Color::new(1.0, 1.0, 0.5, 1.0) // 金黄色
                } else { 
                    WHITE 
                };
                
                draw_texture_ex(
                    texture,
                    draw_x,
                    draw_y,
                    tint_color,
                    DrawTextureParams {
                        dest_size: Some(macroquad::math::Vec2::new(size, size)),
                        rotation: 0.0,
                        flip_x: false,
                        flip_y: false,
                        pivot: None,
                        source: None,
                    }
                );
                
                // 为激光添加额外的光线效果
                if bullet.bullet_type == BulletType::PlayerLaser {
                    // 绘制拖尾效果
                    draw_circle(bullet.position.x, bullet.position.y + 10.0, 2.0, Color::new(0.5, 0.8, 1.0, 0.5));
                    draw_circle(bullet.position.x, bullet.position.y + 20.0, 1.0, Color::new(0.5, 0.8, 1.0, 0.3));
                }
            } else {
                // 回退到原来的圆形绘制
                // 检查是否为激光子弹（穿透计数为9999）
                if bullet.piercing_count == 9999 {
                    // 激光子弹 - 绘制更长的射线效果
                    let color = if bullet.is_crit { YELLOW } else { SKYBLUE };
                    draw_circle(bullet.position.x, bullet.position.y, 4.0, color);
                    // 绘制拖尾效果
                    draw_circle(bullet.position.x, bullet.position.y + 10.0, 2.0, Color::new(color.r, color.g, color.b, 0.5));
                    draw_circle(bullet.position.x, bullet.position.y + 20.0, 1.0, Color::new(color.r, color.g, color.b, 0.3));
                } else {
                    // 普通子弹
                    let color = if bullet.is_crit { YELLOW } else { WHITE };
                    draw_circle(bullet.position.x, bullet.position.y, 3.0, color);
                }
            }
        } else {
            // 敌人子弹 - 根据子弹类型选择纹理
            let (texture_opt, size) = match bullet.bullet_type {
                BulletType::EnemyHeavy => (&game.heavy_bullet_texture, 6.0),
                BulletType::EnemyBoss => (&game.boss_bullet_texture, 8.0),
                BulletType::EnemyGeneric => (&None, 2.0), // 普通敌人子弹使用圆形
                _ => (&None, 2.0), // 默认情况 - 修复类型匹配
            };
            
            if let Some(texture) = texture_opt {
                // 使用纹理绘制
                let draw_x = bullet.position.x - size / 2.0;
                let draw_y = bullet.position.y - size / 2.0;
                
                draw_texture_ex(
                    texture,
                    draw_x,
                    draw_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(macroquad::math::Vec2::new(size, size)),
                        rotation: 0.0,
                        flip_x: false,
                        flip_y: false,
                        pivot: None,
                        source: None,
                    }
                );
            } else {
                // 敌人子弹使用橙色圆形
                draw_circle(bullet.position.x, bullet.position.y, 2.0, ORANGE);
            }
        }
    }
    
    // 绘制道具
    for item in &game.items {
        match item.item_type {
            ItemType::HealthPack => {
                if let Some(texture) = &game.health_pack_texture {
                    // 使用纹理绘制回血道具
                    let size = 20.0;
                    let draw_x = item.position.x - size / 2.0;
                    let draw_y = item.position.y - size / 2.0;
                    
                    // 添加轻微的闪烁效果
                    let pulse = (get_time() as f32 * 5.0).sin() * 0.2 + 0.8;
                    let tint_color = Color::new(1.0, 1.0, 1.0, pulse);
                    
                    draw_texture_ex(
                        texture,
                        draw_x,
                        draw_y,
                        tint_color,
                        DrawTextureParams {
                            dest_size: Some(macroquad::math::Vec2::new(size, size)),
                            rotation: 0.0,
                            flip_x: false,
                            flip_y: false,
                            pivot: None,
                            source: None,
                        }
                    );
                } else {
                    // 回退到绿色圆形绘制
                    draw_circle(item.position.x, item.position.y, 10.0, GREEN);
                }
            }
        }
    }
}

// ==================== 渲染UI ====================

fn render_ui(game: &Game) {
    let font_size = 20.0;
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;
    
    match game.state {
        GameState::MainMenu => {
            // 游戏标题
            draw_text(
                "Puppy WCP And His Four Dom Masters",
                center_x - 200.0,
                center_y - 150.0,
                32.0,
                YELLOW
            );
            
            // 用户状态显示
            if game.user.is_logged_in {
                draw_text(
                    &format!("WELCOME, {}", game.user.username),
                    center_x - 60.0,
                    center_y - 100.0,
                    18.0,
                    GREEN
                );
            } else {
                draw_text(
                    "NOT LOGGED IN",
                    center_x - 50.0,
                    center_y - 100.0,
                    18.0,
                    RED
                );
            }
            
            // 菜单选项
            draw_text("1. Start Game", center_x - 50.0, center_y - 40.0, font_size, WHITE);
            draw_text("2. Login", center_x - 50.0, center_y - 10.0, font_size, WHITE);
            
            // 操作提示
            draw_text("Press 1-2 to select", center_x - 70.0, center_y + 80.0, 16.0, LIGHTGRAY);
        },
        GameState::WeaponSelect => {
            // 标题
            draw_text(
                "Weapon Selection",
                center_x - 60.0,
                center_y - 150.0,
                32.0,
                YELLOW
            );
            
            // 武器选项
            draw_text("1. Machinegun", center_x - 200.0, center_y - 80.0, font_size, WHITE);
            draw_text("Damage:2", center_x - 200.0, center_y - 55.0, 16.0, GRAY);
            draw_text("Attack speed:high", center_x - 200.0, center_y - 35.0, 16.0, GRAY);
            
            draw_text("2. Laser", center_x - 30.0, center_y - 80.0, font_size, WHITE);
            draw_text("Damage:4", center_x - 30.0, center_y - 55.0, 16.0, GRAY);
            draw_text("Attack speed:0.8s", center_x - 30.0, center_y - 35.0, 16.0, GRAY);
            
            draw_text("3. Shotgun", center_x + 140.0, center_y - 80.0, font_size, WHITE);
            draw_text("Damage:4", center_x + 140.0, center_y - 55.0, 16.0, GRAY);
            draw_text("Attack speed:medium", center_x + 140.0, center_y - 35.0, 16.0, GRAY);
            
            // 操作提示
            draw_text("Press 1-3 to select weapon", center_x - 80.0, center_y + 60.0, 16.0, LIGHTGRAY);
            draw_text("ESC to return to main menu", center_x - 80.0, center_y + 80.0, 16.0, LIGHTGRAY);
        },
        GameState::Login => {
            // 标题
            draw_text(
                "User Login",
                center_x - 60.0,
                center_y - 150.0,
                32.0,
                YELLOW
            );
            
            // 输入框
            match game.input_mode {
                InputMode::Username => {
                    draw_text("Username:", center_x - 100.0, center_y - 60.0, font_size, WHITE);
                    let input_display = if game.input_text.is_empty() { "_" } else { &game.input_text };
                    draw_text(&format!("> {}", input_display), center_x - 100.0, center_y - 30.0, 18.0, GREEN);
                    
                    draw_text("Please enter your username", center_x - 80.0, center_y + 20.0, 16.0, LIGHTGRAY);
                },
                InputMode::Password => {
                    draw_text("Username:", center_x - 100.0, center_y - 60.0, font_size, WHITE);
                    draw_text(&format!("> {}", game.user.username), center_x - 100.0, center_y - 30.0, 18.0, GRAY);
                    
                    draw_text("Password:", center_x - 100.0, center_y + 10.0, font_size, WHITE);
                    let password_display = "*".repeat(game.input_text.len()) + "_";
                    draw_text(&format!("> {}", password_display), center_x - 100.0, center_y + 40.0, 18.0, GREEN);
                    
                    draw_text("Please enter your password", center_x - 70.0, center_y + 80.0, 16.0, LIGHTGRAY);
                },
                _ => {}
            }
            
            // 提示信息
            draw_text("Test account: admin", center_x - 80.0, center_y + 120.0, 14.0, DARKGRAY);
            draw_text("Test password: 123456", center_x - 80.0, center_y + 140.0, 14.0, DARKGRAY);
            draw_text("Enter|ESC", center_x - 100.0, center_y + 170.0, 16.0, LIGHTGRAY);
        },
        GameState::Battle => {
            // 玩家状态
            draw_text(&format!("HP: {}/{}", game.player.health, game.player.max_health), 10.0, 30.0, font_size, WHITE);
            draw_text(&format!("LV: {}", game.player.level), 10.0, 55.0, font_size, WHITE);
            draw_text(&format!("EXP: {}/{}", game.player.experience, game.player.experience_needed), 10.0, 80.0, font_size, WHITE);
            
            // 本局统计（显示实时数据）
            draw_text(&format!("Coins: {}", game.current_session_coins), 10.0, 105.0, font_size, YELLOW);
            draw_text(&format!("Enemies: {}", game.enemies_defeated_this_session), 10.0, 130.0, font_size, ORANGE);
            
            // 无敌状态显示
            if game.player.last_damage_time.elapsed().as_secs_f32() < game.player.invincibility_duration {
                let remaining_time = game.player.invincibility_duration - game.player.last_damage_time.elapsed().as_secs_f32();
                draw_text(&format!("Invincible: {:.1}s", remaining_time), 10.0, 155.0, font_size, SKYBLUE);
            }
            
            // 游戏时间
            let time = game.get_game_time();
            let minutes = (time / 60.0) as i32;
            let seconds = (time % 60.0) as i32;
            draw_text(&format!("Time: {}:{:02}", minutes, seconds), 10.0, 180.0, font_size, WHITE);
            
            // 敌人和子弹数量
            draw_text(&format!("Enemies: {}", game.enemies.len()), 10.0, 205.0, font_size, RED);
            draw_text(&format!("Bullets: {}", game.bullets.len()), 10.0, 230.0, font_size, WHITE);
            
            // 武器信息
            let weapon_name = match game.player.weapon.weapon_type {
                WeaponType::MachineGun => "Machinegun",
                WeaponType::Laser => "Laser",
                WeaponType::Shotgun => "Shotgun",
            };
            draw_text(&format!("Weapon: {}", weapon_name), 10.0, 255.0, font_size, BLUE);
            
            // Boss血条显示
            if let Some(boss) = game.enemies.iter().find(|e| e.enemy_type == EnemyType::Boss) {
                let bar_width = 400.0;
                let bar_height = 20.0;
                let bar_x = center_x - bar_width / 2.0;
                let bar_y = screen_height() - 60.0;
                
                // Boss名称
                draw_text("Star Destroyer", center_x - 30.0, bar_y - 10.0, 20.0, RED);
                
                // 血条背景
                draw_rectangle(bar_x, bar_y, bar_width, bar_height, DARKGRAY);
                
                // 血条
                let health_ratio = boss.health as f32 / boss.max_health as f32;
                let health_color = if health_ratio > 0.6 { 
                    RED 
                } else if health_ratio > 0.3 { 
                    ORANGE 
                } else { 
                    MAROON 
                };
                draw_rectangle(bar_x, bar_y, bar_width * health_ratio, bar_height, health_color);
                
                // 血量数字
                draw_text(
                    &format!("{}/{}", boss.health, boss.max_health), 
                    center_x - 30.0, 
                    bar_y + 35.0, 
                    16.0, 
                    WHITE
                );
            }
            
            // 控制说明
            draw_text("WASD: Move", 10.0, screen_height() - 80.0, 16.0, LIGHTGRAY);
            draw_text("Auto-shoot", 10.0, screen_height() - 60.0, 16.0, LIGHTGRAY);
            draw_text("ESC: Return to main menu", 10.0, screen_height() - 40.0, 16.0, LIGHTGRAY);
        },
        GameState::RogueSelection => {
            // 使用新的卡片式界面
            render_rogue_selection_cards(game, center_x, center_y);
        },
        GameState::GameOver => {
            if let Some(result) = game.get_game_result() {
                // 显示结算标题
                let title = if result.victory { "Victory!" } else { "Game Over!" };
                let title_color = if result.victory { GREEN } else { RED };
                draw_text(title, center_x - 30.0, center_y - 150.0, 32.0, title_color);
                
                // 生存时间
                let minutes = (result.survival_time / 60.0) as i32;
                let seconds = (result.survival_time % 60.0) as i32;
                draw_text(
                    &format!("Survival Time: {}m{}s", minutes, seconds),
                    center_x - 120.0,
                    center_y - 70.0,
                    18.0,
                    WHITE
                );
                
                // 使用的武器
                let weapon_name = match result.weapon_used {
                    WeaponType::MachineGun => "Machinegun",
                    WeaponType::Laser => "Laser",
                    WeaponType::Shotgun => "Shotgun",
                };
                draw_text(
                    &format!("Weapon: {}", weapon_name),
                    center_x - 120.0,
                    center_y - 50.0,
                    18.0,
                    BLUE
                );
                
                // 等级和击败敌人数
                draw_text(
                    &format!("Final Level: {}", result.final_level),
                    center_x - 120.0,
                    center_y - 30.0,
                    18.0,
                    WHITE
                );
                
                draw_text(
                    &format!("Enemies: {}", result.enemies_defeated),
                    center_x - 120.0,
                    center_y - 10.0,
                    18.0,
                    WHITE
                );
                
                // 造成伤害
                draw_text(
                    &format!("Damage: {}", result.total_damage_dealt),
                    center_x - 120.0,
                    center_y + 10.0,
                    18.0,
                    WHITE
                );
                
                // 获得的金币和经验
                draw_text(
                    &format!("Coins: {}", result.coins_earned),
                    center_x - 120.0,
                    center_y + 30.0,
                    18.0,
                    YELLOW
                );
                
                draw_text(
                    &format!("Experience: {}", result.experience_gained),
                    center_x - 120.0,
                    center_y + 50.0,
                    18.0,
                    YELLOW
                );
                
                // 操作提示
                draw_text("--- All progress has been cleared, restart ---", center_x - 120.0, center_y + 90.0, 16.0, ORANGE);
                draw_text("R Re-select weapon", center_x - 80.0, center_y + 110.0, 18.0, LIGHTGRAY);
                draw_text("ESC Return to main menu", center_x - 80.0, center_y + 130.0, 18.0, LIGHTGRAY);
            } else {
                // 如果没有结算数据，显示默认信息
                draw_text("Game Over!", center_x - 80.0, center_y - 40.0, 32.0, RED);
                draw_text("R Re-select weapon", center_x - 80.0, center_y + 70.0, 18.0, LIGHTGRAY);
                draw_text("ESC Return to main menu", center_x - 80.0, center_y + 90.0, 18.0, LIGHTGRAY);
            }
        },
    }
}

// ==================== 渲染肉鸽升级卡片界面 ====================

fn render_rogue_selection_cards(game: &Game, center_x: f32, center_y: f32) {
    // 背景半透明遮罩
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
    
    // 主标题 - 居中显示
    let title_text = "Choose one";
    let title_width = measure_text(title_text, None, 28, 1.0).width;
    draw_text(
        title_text,
        center_x - title_width / 2.0,
        center_y - 180.0,
        28.0,
        Color::new(1.0, 0.9, 0.4, 1.0) // 金色标题
    );
    
    // 倒计时显示 - 居中显示
    if !game.rogue_auto_selected {
        let remaining_time = (10.0 - game.rogue_selection_timer).max(0.0);
        let timer_color = if remaining_time <= 3.0 { 
            Color::new(1.0, 0.3, 0.3, 1.0) // 红色警告
        } else { 
            Color::new(0.8, 0.8, 0.8, 1.0) // 灰白色
        };
        
        let timer_text = &format!("Remaining time: {:.1}s", remaining_time);
        let timer_width = measure_text(timer_text, None, 20, 1.0).width;
        draw_text(
            timer_text,
            center_x - timer_width / 2.0,
            center_y - 140.0,
            20.0,
            timer_color
        );
    } else {
        let auto_text = "Time's up! Auto-selected";
        let auto_width = measure_text(auto_text, None, 20, 1.0).width;
        draw_text(
            auto_text,
            center_x - auto_width / 2.0,
            center_y - 140.0,
            20.0,
            Color::new(1.0, 0.6, 0.2, 1.0) // 橙色
        );
    }
    
    // 卡片布局参数
    let card_width = 180.0;
    let card_height = 220.0;
    let card_spacing = 20.0;
    let total_width = game.current_rogue_options.len() as f32 * card_width + 
                     (game.current_rogue_options.len() - 1) as f32 * card_spacing;
    let start_x = center_x - total_width / 2.0;
    
    // 绘制每个升级卡片
    for (i, upgrade) in game.current_rogue_options.iter().enumerate() {
        let card_x = start_x + i as f32 * (card_width + card_spacing);
        let card_y = center_y - card_height / 2.0;
        
        // 确定卡片状态和颜色
        let (card_color, border_color, is_selected) = if game.rogue_auto_selected {
            if let Some(last_upgrade) = game.player.rogue_upgrades.last() {
                if last_upgrade.id == upgrade.id {
                    // 被选中的卡片
                    (Color::new(0.3, 0.6, 0.3, 0.9), Color::new(0.4, 1.0, 0.4, 1.0), true)
                } else {
                    // 未被选中的卡片（变暗）
                    (Color::new(0.2, 0.2, 0.2, 0.7), Color::new(0.4, 0.4, 0.4, 0.8), false)
                }
            } else {
                (Color::new(0.2, 0.3, 0.4, 0.8), upgrade.get_rarity_color(), false)
            }
        } else {
            // 正常状态
            (Color::new(0.2, 0.3, 0.4, 0.8), upgrade.get_rarity_color(), false)
        };
        
        // 绘制卡片背景
        draw_rectangle(card_x, card_y, card_width, card_height, card_color);
        
        // 绘制卡片边框（稀有度颜色）
        let border_thickness = if is_selected { 4.0 } else { 2.0 };
        draw_rectangle_lines(card_x, card_y, card_width, card_height, border_thickness, border_color);
        
        // 绘制稀有度装饰条
        let decoration_height = 8.0;
        draw_rectangle(card_x, card_y, card_width, decoration_height, border_color);
        
        // 绘制大图标 - 居中显示
        let icon_size = 32.0;
        let icon_width = measure_text(&upgrade.icon, None, icon_size as u16, 1.0).width;
        draw_text(
            &upgrade.icon,
            card_x + (card_width - icon_width) / 2.0,
            card_y + 50.0,
            icon_size,
            if is_selected { Color::new(1.0, 1.0, 0.5, 1.0) } else { WHITE }
        );
        
        // 绘制升级名称 - 居中显示
        let name_color = if is_selected { 
            Color::new(1.0, 1.0, 0.5, 1.0) 
        } else { 
            upgrade.get_rarity_color() 
        };
        let name_width = measure_text(&upgrade.name, None, 18, 1.0).width;
        draw_text(
            &upgrade.name,
            card_x + (card_width - name_width) / 2.0,
            card_y + 85.0,
            18.0,
            name_color
        );
        
        // 绘制简短描述 - 居中显示
        let short_desc_width = measure_text(&upgrade.short_desc, None, 16, 1.0).width;
        draw_text(
            &upgrade.short_desc,
            card_x + (card_width - short_desc_width) / 2.0,
            card_y + 110.0,
            16.0,
            Color::new(1.0, 0.9, 0.3, 1.0) // 金黄色
        );
        
        // 绘制详细描述（自动换行并居中）
        let desc_lines = wrap_text(&upgrade.detailed_desc, 22); // 每行约22个字符
        for (line_idx, line) in desc_lines.iter().enumerate() {
            let line_width = measure_text(line, None, 14, 1.0).width;
            draw_text(
                line,
                card_x + (card_width - line_width) / 2.0,
                card_y + 140.0 + line_idx as f32 * 18.0,
                14.0,
                Color::new(0.9, 0.9, 0.9, 1.0) // 浅灰色
            );
        }
        
        // 绘制选择提示数字
        if !game.rogue_auto_selected {
            let number_bg_size = 25.0;
            let number_x = card_x + card_width - number_bg_size - 5.0;
            let number_y = card_y + 5.0;
            
            // 数字背景圆圈
            draw_circle(
                number_x + number_bg_size / 2.0, 
                number_y + number_bg_size / 2.0, 
                number_bg_size / 2.0, 
                Color::new(0.1, 0.1, 0.1, 0.8)
            );
            
            // 数字边框
            draw_circle_lines(
                number_x + number_bg_size / 2.0, 
                number_y + number_bg_size / 2.0, 
                number_bg_size / 2.0, 
                2.0, 
                upgrade.get_rarity_color()
            );
            
            // 数字文本 - 居中显示
            let number_text = &(i + 1).to_string();
            let number_width = measure_text(number_text, None, 18, 1.0).width;
            draw_text(
                number_text,
                number_x + (number_bg_size - number_width) / 2.0,
                number_y + 18.0,
                18.0,
                WHITE
            );
        }
    }
    
    // 底部操作提示 - 居中显示
    if !game.rogue_auto_selected {
        let select_text = "Press 1-3 to select";
        let select_width = measure_text(select_text, None, 18, 1.0).width;
        draw_text(
            select_text,
            center_x - select_width / 2.0,
            center_y + 150.0,
            18.0,
            Color::new(0.8, 0.8, 0.8, 1.0)
        );
        
        let timeout_text = "Timeout will automatically select";
        let timeout_width = measure_text(timeout_text, None, 16, 1.0).width;
        draw_text(
            timeout_text,
            center_x - timeout_width / 2.0,
            center_y + 175.0,
            16.0,
            Color::new(1.0, 0.6, 0.2, 1.0)
        );
    } else {
        let remaining_delay = 2.0 - game.rogue_auto_selected_timer;
        let return_text = &format!("{:.1}s return to battle...", remaining_delay.max(0.0));
        let return_width = measure_text(return_text, None, 18, 1.0).width;
        draw_text(
            return_text,
            center_x - return_width / 2.0,
            center_y + 150.0,
            18.0,
            Color::new(1.0, 0.8, 0.3, 1.0)
        );
    }
}

// ==================== 文本自动换行辅助函数 ====================

fn wrap_text(text: &str, max_chars_per_line: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for char in text.chars() {
        current_line.push(char);
        if current_line.len() >= max_chars_per_line || char == '\n' {
            lines.push(current_line.clone());
            current_line.clear();
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_player_creation() {
        let player = Player::new();
        assert_eq!(player.health, 20);
        assert_eq!(player.level, 1);
        assert_eq!(player.experience, 0);
    }
    
    #[test]
    fn test_weapon_enhancement() {
        let mut weapon = Weapon::new(WeaponType::MachineGun);
        weapon.enhancement_level = 5;
        assert_eq!(weapon.get_total_attack_power(), 7);
    }
    
    #[test]
    fn test_enemy_damage() {
        let mut enemy = Enemy::new(EnemyType::Scout, Vec2::new(100.0, 100.0));
        enemy.take_damage(10);
        assert_eq!(enemy.health, 10);
    }
    
    #[test]
    fn test_level_up() {
        let mut player = Player::new();
        player.add_experience(100);
        assert_eq!(player.level, 2);
        assert_eq!(player.experience, 0);
        assert_eq!(player.experience_needed, 200);
    }
}