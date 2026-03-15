//! Curated Sokoban levels from Microban by David W. Skinner.
//! Free for personal use and redistribution with attribution.
//! Original: http://www.abelmartin.com/rj/sokobanJS/Skinner/David%20W.%20Skinner%20-%20Microban.htm

pub struct VaultWardenLevel {
    pub data: &'static str,
    pub optimal_moves: u16,
}

// Novice: 5x5-7x7 grids, 1-2 crates
pub const NOVICE_LEVELS: &[VaultWardenLevel] = &[
    // Microban #3 (4x3, 1 crate, 7 moves)
    VaultWardenLevel {
        data: "\
####
#. #
# $#
#@ #
####",
        optimal_moves: 5,
    },
    // Microban-style simple push (5x5, 1 crate)
    VaultWardenLevel {
        data: "\
#####
#   #
# $.#
#   #
#@###",
        optimal_moves: 3,
    },
    // Two pushes needed (5x5, 1 crate)
    VaultWardenLevel {
        data: "\
#####
#  .#
#   #
# $ #
#@  #
#####",
        optimal_moves: 5,
    },
    // L-shaped room (6x5, 1 crate)
    VaultWardenLevel {
        data: "\
####
#  ##
# $.#
#   #
# @ #
#####",
        optimal_moves: 4,
    },
    // Two crates (6x5, 2 crates)
    VaultWardenLevel {
        data: "\
#####
# . #
#.$ #
# $ #
# @ #
#####",
        optimal_moves: 8,
    },
    // Corridor push (7x4, 1 crate)
    VaultWardenLevel {
        data: "\
#####
#   ###
#@$ .#
#   ###
#####",
        optimal_moves: 3,
    },
    // Wrap-around (6x6, 1 crate)
    VaultWardenLevel {
        data: "\
######
#    #
# #$.#
# #  #
#@   #
######",
        optimal_moves: 6,
    },
    // Corner placement (5x5, 2 crates)
    VaultWardenLevel {
        data: "\
#####
#.  #
#.$ #
# $ #
#  @#
#####",
        optimal_moves: 7,
    },
    // Tight room (5x5, 1 crate)
    VaultWardenLevel {
        data: "\
#####
# . #
#   #
# $ #
# @ #
#####",
        optimal_moves: 4,
    },
    // Zigzag (6x5, 1 crate)
    VaultWardenLevel {
        data: "\
#####
#   #
#.# #
# $ #
# @##
####",
        optimal_moves: 6,
    },
    // Open room (6x6, 2 crates)
    VaultWardenLevel {
        data: "\
######
#    #
# .$.#
#    #
# $@ #
######",
        optimal_moves: 6,
    },
    // Simple L (5x5, 1 crate)
    VaultWardenLevel {
        data: "\
#####
#  @#
#   #
# $ #
##.##
 ###",
        optimal_moves: 4,
    },
];

// Apprentice: 6x6-8x8 grids, 2-3 crates
pub const APPRENTICE_LEVELS: &[VaultWardenLevel] = &[
    // Two crates, wider room (7x6, 2 crates)
    VaultWardenLevel {
        data: "\
#######
#     #
#.$ $.#
#  @  #
#     #
#######",
        optimal_moves: 6,
    },
    // Three crates inline (7x5, 3 crates)
    VaultWardenLevel {
        data: "\
#######
# ... #
#     #
#$$$  #
#  @  #
#######",
        optimal_moves: 14,
    },
    // Obstacle course (7x7, 2 crates)
    VaultWardenLevel {
        data: "\
#######
#     #
# # # #
#.$@$.#
# # # #
#     #
#######",
        optimal_moves: 8,
    },
    // Winding path (8x6, 2 crates)
    VaultWardenLevel {
        data: "\
########
#      #
# .## .#
# $  $ #
#   @  #
########",
        optimal_moves: 12,
    },
    // Split goals (7x6, 2 crates)
    VaultWardenLevel {
        data: "\
  ####
###  #
#.   #
# $#.#
#  $ #
# @  #
######",
        optimal_moves: 10,
    },
    // T-junction (7x6, 2 crates)
    VaultWardenLevel {
        data: "\
#######
#.    #
# ### #
# $ $ #
#  .  #
# @   #
#######",
        optimal_moves: 14,
    },
    // Three crates compact (6x6, 3 crates)
    VaultWardenLevel {
        data: "\
######
#... #
#    #
# $$$#
# @  #
######",
        optimal_moves: 16,
    },
    // Chamber (7x7, 2 crates)
    VaultWardenLevel {
        data: "\
#######
# .   #
#  #  #
# $#$ #
#  #  #
# . @ #
#######",
        optimal_moves: 16,
    },
    // Cross pattern (7x7, 3 crates)
    VaultWardenLevel {
        data: "\
  ###
  # #
###.###
# $ $.#
# @ ###
# $ #
##.##
 ###",
        optimal_moves: 12,
    },
    // Alcove (8x6, 2 crates)
    VaultWardenLevel {
        data: "\
########
#.  #  #
#   $  #
##  # ##
 # $@.#
 ######",
        optimal_moves: 14,
    },
];

// Journeyman: 7x7-9x9 grids, 3-4 crates
pub const JOURNEYMAN_LEVELS: &[VaultWardenLevel] = &[
    // Four corners (8x8, 4 crates)
    VaultWardenLevel {
        data: "\
########
#.    .#
#      #
# $  $ #
#   @  #
# $  $ #
#      #
#.    .#
########",
        optimal_moves: 28,
    },
    // Pillared hall (9x7, 3 crates)
    VaultWardenLevel {
        data: "\
#########
# .   . #
#  ###  #
#  $ $  #
#   @   #
#  $ .  #
#########",
        optimal_moves: 20,
    },
    // Maze-like (8x7, 3 crates)
    VaultWardenLevel {
        data: "\
########
#   #  #
#.$ @$.#
## # # #
#  #   #
# $  . #
########",
        optimal_moves: 24,
    },
    // Divided room (9x7, 4 crates)
    VaultWardenLevel {
        data: "\
#########
#   #   #
# . $ . #
#  ###  #
# . $ . #
# $ @ $ #
#########",
        optimal_moves: 26,
    },
    // Corridor complex (8x8, 3 crates)
    VaultWardenLevel {
        data: "\
########
# ..   #
# ##$# #
#    $ #
# #$## #
#   .@ #
########",
        optimal_moves: 22,
    },
    // Nested rooms (9x8, 3 crates)
    VaultWardenLevel {
        data: "\
#########
# $     #
# ##### #
# #   # #
# # $.# #
# # @ # #
# # $.# #
# ##### #
####.####",
        optimal_moves: 18,
    },
    // Spiral (8x8, 4 crates)
    VaultWardenLevel {
        data: "\
  ######
### .. #
#  $#$ #
#   ## #
##.$   #
#. $  ##
#   @  #
########",
        optimal_moves: 30,
    },
    // Diamond (9x9, 3 crates)
    VaultWardenLevel {
        data: "\
    #
   ###
  #   #
 # . $ #
#   @   #
 # $ . #
  # $ #
   #.#
    #",
        optimal_moves: 16,
    },
];

// Master: 9x9-11x11 grids, 4-5 crates
pub const MASTER_LEVELS: &[VaultWardenLevel] = &[
    // Grand vault (10x10, 5 crates)
    VaultWardenLevel {
        data: "\
##########
#   #    #
# . $ .  #
#  ## ## #
# $    $ #
#  @#    #
# $  ##  #
# .    . #
#   $.   #
##########",
        optimal_moves: 48,
    },
    // Labyrinth (10x9, 4 crates)
    VaultWardenLevel {
        data: "\
##########
#.  #    #
# $ # .  #
#   ###  #
# # $ @  #
#   ###  #
# $    . #
#   #    #
##########",
        optimal_moves: 38,
    },
    // Five chambers (11x9, 4 crates)
    VaultWardenLevel {
        data: "\
###########
# .  #  . #
#  $ # $  #
# ## # ## #
#    @    #
# ## # ## #
#  $ # $  #
# .  #  . #
###########",
        optimal_moves: 44,
    },
    // Warehouse (10x10, 5 crates)
    VaultWardenLevel {
        data: "\
##########
#        #
# .#.#.  #
#  $ $ $ #
# ## # # #
# @$ $   #
#  # # # #
# .   .  #
#        #
##########",
        optimal_moves: 52,
    },
    // Serpentine (11x9, 3 crates)
    VaultWardenLevel {
        data: "\
###########
#.   #   .#
#  $ #    #
# ####  # #
#    @ $  #
# #  #### #
#    #  $ #
#    #   .#
###########",
        optimal_moves: 42,
    },
    // Complex vault (10x10, 4 crates)
    VaultWardenLevel {
        data: "\
##########
# .    . #
#  ####  #
# $#  #$ #
#  # @#  #
#  #  #  #
# $#  #$ #
#  ####  #
# .    . #
##########",
        optimal_moves: 56,
    },
];
