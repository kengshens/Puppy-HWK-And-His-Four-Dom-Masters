use macroquad::prelude::*;

#[macroquad::main("Roguelike")]
async fn main() {
    loop {
        clear_background(BLACK);
        
        // 绘制玩家
        draw_circle(400.0, 550.0, 15.0, BLUE);
        
        // 绘制一些测试敌人
        draw_circle(200.0, 100.0, 8.0, RED);
        draw_circle(600.0, 150.0, 12.0, Color::new(0.5, 0.0, 0.0, 1.0)); // DARKRED 替代
        
        // 绘制UI
        draw_text("Roguelike Test", 10.0, 30.0, 24.0, WHITE);
        draw_text("WASD to move (not implemented yet)", 10.0, 60.0, 16.0, LIGHTGRAY);
        
        next_frame().await
    }
} 