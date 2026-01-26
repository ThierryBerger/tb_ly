use bevy::prelude::*;

use crate::game::WallBundle;

pub fn create_map_1(mut commands: Commands) {
    const WALL_SIZE: f32 = 3000.0;

    commands.spawn(WallBundle::new(
        Vec2::new(-WALL_SIZE, -WALL_SIZE),
        Vec2::new(-WALL_SIZE, WALL_SIZE),
        Color::WHITE,
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(-WALL_SIZE, WALL_SIZE),
        Vec2::new(WALL_SIZE, WALL_SIZE),
        Color::WHITE,
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(WALL_SIZE, WALL_SIZE),
        Vec2::new(WALL_SIZE, -WALL_SIZE),
        Color::WHITE,
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(WALL_SIZE, -WALL_SIZE),
        Vec2::new(-WALL_SIZE, -WALL_SIZE),
        Color::WHITE,
    ));
}

pub fn create_map_2(mut commands: Commands) {
    const WALL_SIZE: f32 = 3000.0;
    const COLOR: Color = Color::WHITE;

    // Helper for local use
    let mut wall = |x1: f32, y1: f32, x2: f32, y2: f32| {
        commands.spawn(WallBundle::new(Vec2::new(x1, y1), Vec2::new(x2, y2), COLOR));
    };

    // --- 1. Outer Boundary ---
    wall(-WALL_SIZE, -WALL_SIZE, -WALL_SIZE, WALL_SIZE);
    wall(-WALL_SIZE, WALL_SIZE, WALL_SIZE, WALL_SIZE);
    wall(WALL_SIZE, WALL_SIZE, WALL_SIZE, -WALL_SIZE);
    wall(WALL_SIZE, -WALL_SIZE, -WALL_SIZE, -WALL_SIZE);

    // --- 2. The Central Cross (Broken for passage) ---
    // Top & Bottom vertical splitters
    wall(0.0, 600.0, 0.0, 1500.0);
    wall(0.0, -600.0, 0.0, -1500.0);
    // Left & Right horizontal splitters
    wall(-600.0, 0.0, -1500.0, 0.0);
    wall(600.0, 0.0, 1500.0, 0.0);

    // --- 3. The "Inner Ring" (Four large corner boxes/rooms) ---
    // These create 4 distinct quadrants with 200-unit wide doorways
    let room_size = 400.0;
    let offset = 800.0;
    for x_sign in [-1.0, 1.0] {
        for y_sign in [-1.0, 1.0] {
            let cx = x_sign * offset;
            let cy = y_sign * offset;
            // Draw a small 3-sided "C" shape facing the center
            wall(
                cx - room_size,
                cy - room_size,
                cx + room_size,
                cy - room_size,
            ); // Horiz
            wall(
                cx - room_size,
                cy - room_size,
                cx - room_size,
                cy + room_size,
            ); // Vert
            wall(
                cx - room_size,
                cy + room_size,
                cx + room_size,
                cy + room_size,
            ); // Horiz
        }
    }

    // --- 4. The "Teeth" (Small cover segments scattered in the open lanes) ---
    // North lane
    wall(-200.0, 2200.0, 200.0, 2200.0);
    wall(0.0, 2200.0, 0.0, 2600.0);

    // South lane
    wall(-200.0, -2200.0, 200.0, -2200.0);
    wall(0.0, -2200.0, 0.0, -2600.0);

    // --- 5. Far Corner "Bunkers" ---
    // Top Right
    wall(2500.0, 2500.0, 2100.0, 2500.0);
    wall(2500.0, 2500.0, 2500.0, 2100.0);

    // Bottom Left
    wall(-2500.0, -2500.0, -2100.0, -2500.0);
    wall(-2500.0, -2500.0, -2500.0, -2100.0);
}

const MAP_DATA: &str = "
############################################################
#                                                          #
#   ##     ###        ####    ##        ####   ###         #
#   #        #        #                 #        #         #
#   #   ##   #            ##            #   ##   #         #
#   #        #                 #        #        #         #
#   #### #####             #####        ###     ##         #
#                                                          #
#            ####   ###            ####   ###              #
#    ######  #        #            #        #              #
#            #   ##   #   ######   #   ##   #   ######     #
#   #        #        #   #    #   #        #   #    #     #
#   #        ####  ####   #    #   ###     ##   #    #     #
#   #    #                #    #                #    #     #
#        ####  ####  ######    ######      ######    #     #
#   #                                                #     #
#        ##########  ######    ##########  ######    #     #
#   #    #                                      #    #     #
#        #    #########   #    #   ########     #    #     #
#   #    #            #   #    #   #                       #
#   ######       ##   #   ######   #   ##                  #
#                                           #      ###     #
#             #########            ####  ####        #     #
#                                                    #     #
#   #####  ###        ##########        #####  ###         #
#            #                          #        #         #
#   #   ##   #            ##                ##             #
#   #        #        #        #        #        #         #
#   ###    ###        ##########        ##########         #
#                                                          #
#            ###   ####            #####   ##              #
#            #        #            #        #              #
#   ######   #   ##   #   #  ###   #   ##   #   ######     #
#   #    #            #   #    #   #        #   #    #     #
#   #    #    #########        #        #####   #    #     #
#        #                     #                #          #
#   #         #####  ######    #    #####  ######    #     #
#   #                                                #     #
#   #    ##########  ######    ###   ####  ######    #     #
#   #                     #    #                     #     #
#   #        ##########   #        ##########        #     #
#   #                     #                          #     #
#   ######       ##       #            ##       ######     #
#                #                                         #
#            #######   ###         ##########              #
#                                                          #
#   ##    ####            ######        #####    #         #
#   #        #        #        #        #        #         #
#       ##   #        #        #        #   ##   #         #
#            #        #        #        #        #         #
#   #####  ###        #####    #        ##    ####         #
#                                                          #
############################################################";

pub fn create_map_from_string(mut commands: Commands) {
    let lines: Vec<&str> = MAP_DATA.trim().lines().collect();
    let rows = lines.len();
    let cols = lines[0].len();
    let tile_size: f32 = 50.0;
    let offset_x = (cols as f32 * tile_size) / 2.0;
    let offset_y = (rows as f32 * tile_size) / 2.0;

    // Grid to keep track of what we've already built into a long wall
    let mut h_used = vec![vec![false; cols]; rows];
    let mut v_used = vec![vec![false; cols]; rows];

    let grid: Vec<Vec<bool>> = lines
        .iter()
        .map(|l| l.chars().map(|c| c == '#').collect())
        .collect();

    // 1. Horizontal Merging
    for y in 0..rows {
        let mut x = 0;
        while x < cols {
            if grid[y][x] && (x + 1 < cols && grid[y][x + 1]) && !h_used[y][x] {
                let start_x = x;
                // Stretch the wall as far right as possible
                while x < cols && grid[y][x] {
                    h_used[y][x] = true;
                    x += 1;
                }
                spawn_wall(
                    &mut commands,
                    start_x,
                    y,
                    x - 1,
                    y,
                    tile_size,
                    offset_x,
                    offset_y,
                );
            } else {
                x += 1;
            }
        }
    }

    // 2. Vertical Merging
    for x in 0..cols {
        let mut y = 0;
        while y < rows {
            if grid[y][x] && (y + 1 < rows && grid[y + 1][x]) && !v_used[y][x] {
                let start_y = y;
                // Stretch the wall as far down as possible
                while y < rows && grid[y][x] {
                    v_used[y][x] = true;
                    y += 1;
                }
                spawn_wall(
                    &mut commands,
                    x,
                    start_y,
                    x,
                    y - 1,
                    tile_size,
                    offset_x,
                    offset_y,
                );
            } else {
                y += 1;
            }
        }
    }
}

fn spawn_wall(
    commands: &mut Commands,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    size: f32,
    off_x: f32,
    off_y: f32,
) {
    let start = Vec2::new(x1 as f32 * size - off_x, off_y - y1 as f32 * size);
    let end = Vec2::new(x2 as f32 * size - off_x, off_y - y2 as f32 * size);

    commands.spawn(WallBundle::new(start, end, Color::WHITE));
}

pub fn create_map_3(mut commands: Commands) {
    create_map_from_string(commands);
}
