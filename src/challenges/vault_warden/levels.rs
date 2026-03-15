//! Curated Sokoban levels from Microban by David W. Skinner.
//! Free for personal use and redistribution with attribution.
//! Original: http://www.abelmartin.com/rj/sokobanJS/Skinner/David%20W.%20Skinner%20-%20Microban.htm

pub struct VaultWardenLevel {
    pub data: &'static str,
}

pub const NOVICE_LEVELS: &[VaultWardenLevel] = &[
    // Level 44
    VaultWardenLevel {
        data: "\
#####
#@$.#
#####",
    },
    // Level 1
    VaultWardenLevel {
        data: "\
####
# .#
#  ###
#*@  #
#  $ #
#  ###
####",
    },
    // Level 9
    VaultWardenLevel {
        data: "\
#####
#.  ##
#@$$ #
##   #
 ##  #
  ##.#
   ###",
    },
    // Level 14
    VaultWardenLevel {
        data: "\
#######
#     #
# # # #
#. $*@#
#   ###
#####",
    },
    // Level 21
    VaultWardenLevel {
        data: "\
####
#  ####
# . . #
# $$#@#
##    #
 ######",
    },
    // Level 23
    VaultWardenLevel {
        data: "\
#######
#  *  #
#     #
## # ##
 #$@.#
 #   #
 #####",
    },
    // Level 24
    VaultWardenLevel {
        data: "\
# #####
  #   #
###$$@#
#   ###
#     #
# . . #
#######",
    },
    // Level 27
    VaultWardenLevel {
        data: "\
######
#   .#
# ## ##
#  $$@#
# #   #
#.  ###
#####",
    },
    // Level 28
    VaultWardenLevel {
        data: "\
#####
#   #
# @ #
# $$###
##. . #
 #    #
 ######",
    },
    // Level 56
    VaultWardenLevel {
        data: "\
#####
#   ###
#  $  #
##* . #
 #   @#
 ######",
    },
    // Level 19
    VaultWardenLevel {
        data: "\
########
#   .. #
#  @$$ #
##### ##
   #  #
   #  #
   #  #
   ####",
    },
    // Level 46
    VaultWardenLevel {
        data: "\
 ######
##    #
#  ## #
# # $ #
#  * .#
## #@##
 #   #
 #####",
    },
    // Level 51
    VaultWardenLevel {
        data: "\
####
#  ###
#    ###
#  $*@ #
### .# #
  #    #
  ######",
    },
    // Level 3
    VaultWardenLevel {
        data: "\
  ####
###  ####
#     $ #
# #  #$ #
# . .#@ #
#########",
    },
    // Level 11
    VaultWardenLevel {
        data: "\
  ######
  #    #
  # ##@##
### # $ #
# ..# $ #
#       #
#  ######
####",
    },
    // Level 12
    VaultWardenLevel {
        data: "\
#####
#   ##
# $  #
## $ ####
 ###@.  #
  #  .# #
  #     #
  #######",
    },
    // Level 15
    VaultWardenLevel {
        data: "\
     ###
######@##
#    .* #
#   #   #
#####$# #
    #   #
    #####",
    },
    // Level 18
    VaultWardenLevel {
        data: "\
#######
#     #
#. .  #
# ## ##
#  $ #
###$ #
  #@ #
  #  #
  ####",
    },
    // Level 20
    VaultWardenLevel {
        data: "\
#######
#     ###
#  @$$..#
#### ## #
  #     #
  #  ####
  #  #
  ####",
    },
    // Level 22
    VaultWardenLevel {
        data: "\
#####
#   ###
#. .  #
#   # #
## #  #
 #@$$ #
 #    #
 #  ###
 ####",
    },
    // Level 57
    VaultWardenLevel {
        data: "\
  ####
  #  #
  #@ #
  #  #
### ####
#    * #
#  $   #
#####. #
    ####",
    },
    // Level 39
    VaultWardenLevel {
        data: "\
#####
#   ####
# # # .#
#    $ ###
### #$.  #
#   #@   #
# # ######
#   #
#####",
    },
    // Level 50
    VaultWardenLevel {
        data: "\
  ####
###  #####
#  $  @..#
# $    # #
### #### #
  #      #
  ########",
    },
    // Level 55
    VaultWardenLevel {
        data: "\
########
# @ #  #
#      #
#####$ #
    #  ###
 ## #$ ..#
 ## #  ###
    ####",
    },
    // Level 2
    VaultWardenLevel {
        data: "\
######
#    #
# #@ #
# $* #
# .* #
#    #
######",
    },
    // Level 17
    VaultWardenLevel {
        data: "\
#####
# @ #
#...#
#$$$##
#    #
#    #
######",
    },
    // Level 25
    VaultWardenLevel {
        data: "\
 ####
 #  ###
 # $$ #
##... #
#  @$ #
#   ###
#####",
    },
    // Level 30
    VaultWardenLevel {
        data: "\
####
#  ###
# $$ #
#... #
# @$ #
#   ##
#####",
    },
    // Level 31
    VaultWardenLevel {
        data: "\
  ####
 ##  #
##@$.##
# $$  #
# . . #
###   #
  #####",
    },
    // Level 32
    VaultWardenLevel {
        data: "\
 ####
##  ###
#     #
#.**$@#
#   ###
##  #
 ####",
    },
    // Level 33
    VaultWardenLevel {
        data: "\
#######
#. #  #
#  $  #
#. $#@#
#  $  #
#. #  #
#######",
    },
    // Level 40
    VaultWardenLevel {
        data: "\
 #####
 #   #
##   ##
# $$$ #
# .+. #
#######",
    },
    // Level 45
    VaultWardenLevel {
        data: "\
######
#... #
#  $ #
# #$##
#  $ #
#  @ #
######",
    },
    // Level 58
    VaultWardenLevel {
        data: "\
####
#  ####
#.*$  #
# .$# #
## @  #
 #   ##
 #####",
    },
    // Level 29
    VaultWardenLevel {
        data: "\
     #####
     #   ##
     #    #
 ######   #
##     #. #
# $ $ @  ##
# ######.#
#        #
##########",
    },
    // Level 47
    VaultWardenLevel {
        data: "\
  #######
###     #
# $ $   #
# ### #####
# @ . .   #
#   ###   #
##### #####",
    },
    // Level 4
    VaultWardenLevel {
        data: "\
########
#      #
# .**$@#
#      #
#####  #
    ####",
    },
    // Level 8
    VaultWardenLevel {
        data: "\
  ######
  # ..@#
  # $$ #
  ## ###
   # #
   # #
#### #
#    ##
# #   #
#   # #
###   #
  #####",
    },
];

pub const APPRENTICE_LEVELS: &[VaultWardenLevel] = &[
    // Level 26
    VaultWardenLevel {
        data: "\
 #####
 # @ #
 #   #
###$ #
# ...#
# $$ #
###  #
  ####",
    },
    // Level 41
    VaultWardenLevel {
        data: "\
#######
#     #
#@$$$ ##
#  #...#
##    ##
 ######",
    },
    // Level 42
    VaultWardenLevel {
        data: "\
   ####
   #  #
   #@ #
####$.#
#   $.#
# # $.#
#    ##
######",
    },
    // Level 48
    VaultWardenLevel {
        data: "\
######
#  @ #
#  # ##
# .#  ##
# .$$$ #
# .#   #
####   #
   #####",
    },
    // Level 67
    VaultWardenLevel {
        data: "\
#####
#   ##
# #  #
#@$*.##
##  . #
 # $# #
 ##   #
  #####",
    },
    // Level 82
    VaultWardenLevel {
        data: "\
#####
#   ###
# .   ##
##*#$  #
# .# $ #
# @## ##
#     #
#######",
    },
    // Level 71
    VaultWardenLevel {
        data: "\
###########
#     #   ###
# $@$ # .  .#
# ## ### ## #
# #       # #
# #   #   # #
# ######### #
#           #
#############",
    },
    // Level 13
    VaultWardenLevel {
        data: "\
####
#. ##
#.@ #
#. $#
##$ ###
 # $  #
 #    #
 #  ###
 ####",
    },
    // Level 37
    VaultWardenLevel {
        data: "\
      ###
##### #.#
#   ###.#
#   $ #.#
# $  $  #
#####@# #
    #   #
    #####",
    },
    // Level 43
    VaultWardenLevel {
        data: "\
     ####
     # @#
     #  #
###### .#
#   $  .#
#  $$# .#
#    ####
###  #
  ####",
    },
    // Level 81
    VaultWardenLevel {
        data: "\
 #####
 #   #
 # . #
## * #
#  *##
#  @##
## $ #
 #   #
 #####",
    },
    // Level 94
    VaultWardenLevel {
        data: "\
#########
# @ #   #
# $ $   #
##$### ##
#  ...  #
#   #   #
######  #
     ####",
    },
    // Level 104
    VaultWardenLevel {
        data: "\
    #####
#####   #
#    $  #
#  $#$#@#
### #   #
  # ... #
  ###  ##
    #  #
    ####",
    },
    // Level 53
    VaultWardenLevel {
        data: "\
 #####
##. .##
# * * #
#  #  #
# $ $ #
## @ ##
 #####",
    },
    // Level 154
    VaultWardenLevel {
        data: "\
 ############################
 #                          #
 # ######################## #
 # #                      # #
 # # #################### # #
 # # #                  # # #
 # # # ################ # # #
 # # # #              # # # #
 # # # # ############ # # # #
 # # # # #            # # # #
 # # # # # ############ # # #
 # # # # #              # # #
 # # # # ################ # #
 # # # #                  # #
##$# # #################### #
#. @ #                      #
#############################",
    },
    // Level 16
    VaultWardenLevel {
        data: "\
 ####
 #  ####
 #     ##
## ##   #
#. .# @$##
#   # $$ #
#  .#    #
##########",
    },
    // Level 38
    VaultWardenLevel {
        data: "\
##########
#        #
# ##.### #
# # $$ . #
# . @$## #
#####    #
    ######",
    },
    // Level 49
    VaultWardenLevel {
        data: "\
######
# @  #
# $# #
# $  #
# $ ##
### ####
 #  #  #
 #...  #
 #     #
 #######",
    },
    // Level 68
    VaultWardenLevel {
        data: "\
 ####
 #  ######
##    $  #
# .# $   #
# .#$#####
# .@ #
######",
    },
    // Level 73
    VaultWardenLevel {
        data: "\
####
#  #####
# $$ $ #
#      #
## ## ##
#...#@#
# ### ##
#      #
#  #   #
########",
    },
    // Level 79
    VaultWardenLevel {
        data: "\
 #########
 #    #  #
## $#$#  #
#  .$.@  #
#  .#    #
##########",
    },
    // Level 119
    VaultWardenLevel {
        data: "\
  ######
  #    ##
 ## ##  #
 # $$ # #
 # @$ # #
 #    # #
#### #  #
#  ... ##
#     ##
#######",
    },
    // Level 5
    VaultWardenLevel {
        data: "\
 #######
 #     #
 # .$. #
## $@$ #
#  .$. #
#      #
########",
    },
    // Level 52
    VaultWardenLevel {
        data: "\
  ####
### @#
#  $ #
#  *.#
#  *.#
#  $ #
###  #
  ####",
    },
    // Level 100
    VaultWardenLevel {
        data: "\
#######
# @#  #
#.$   #
#. # $##
#.$#   #
#. # $ #
#  #   #
########",
    },
    // Level 103
    VaultWardenLevel {
        data: "\
  #####
  # . ##
### $  #
# . $#@#
# #$ . #
#  $ ###
## . #
 #####",
    },
    // Level 10
    VaultWardenLevel {
        data: "\
      #####
      #.  #
      #.# #
#######.# #
# @ $ $ $ #
# # # # ###
#       #
#########",
    },
    // Level 69
    VaultWardenLevel {
        data: "\
####  ####
#  ####  #
#  #  #  #
#  #    $##
#  . .#$  #
#@ ## # $ #
#   . #   #
###########",
    },
    // Level 72
    VaultWardenLevel {
        data: "\
  ####
 ##  #####
 #  $  @ #
 #  $#   #
#### #####
#  #   #
#    $ #
# ..#  #
#  .####
#  ##
####",
    },
    // Level 76
    VaultWardenLevel {
        data: "\
 #####
##   #
#    #####
#  #.#   #
#@ #.# $ #
#  #.#  ##
#    #  #
##  ##$$#
 ##     #
  #  ####
  ####",
    },
    // Level 85
    VaultWardenLevel {
        data: "\
       ####
 #######  #
 # $      #
 #   $ $  #
 # ########
## # .  #
#  # #  #
#  @ . ##
## # # #
 #   . #
 #######",
    },
    // Level 96
    VaultWardenLevel {
        data: "\
  ######
  #    #
  #    #
#####  #
#   #.#####
#   $@$   #
#####.#   #
   ## ## ##
   #   $.#
   #   ###
   #####",
    },
    // Level 6
    VaultWardenLevel {
        data: "\
###### #####
#    ###   #
# $$     #@#
# $ #...   #
#   ########
#####",
    },
    // Level 34
    VaultWardenLevel {
        data: "\
  ####
###  ####
#       #
#@$***. #
#       #
#########",
    },
    // Level 65
    VaultWardenLevel {
        data: "\
  ######
  #    #
  #  $ #
 ####$ #
## $ $ #
#....# ##
#     @ #
##  #   #
 ########",
    },
    // Level 84
    VaultWardenLevel {
        data: "\
########
#  ... #
#  ### ##
#  # $  #
## #@$  #
 # # $  #
 # ### #####
 #         #
 #   ###   #
 ##### #####",
    },
    // Level 86
    VaultWardenLevel {
        data: "\
    ####
  ###  ##
 ## $   #
## $  # #
# @#$$  #
# ..  ###
# ..###
#####",
    },
    // Level 88
    VaultWardenLevel {
        data: "\
     ####
 # ###  #
 # #    #
 # #  # #
 # #$ #.#
 # #  # # #
 # #$ #.# #
   #  # # #
####$ #.# #
# @     # #
#   #  ## #
########",
    },
];

pub const JOURNEYMAN_LEVELS: &[VaultWardenLevel] = &[
    // Level 110
    VaultWardenLevel {
        data: "\
  ####
  #  #
  # $####
###. .  #
# $ # $ #
#  . .###
####$ #
   # @#
   ####",
    },
    // Level 136
    VaultWardenLevel {
        data: "\
 #######
 #     #
## ###$##
#.$   @ #
# .. #$ #
#.##  $ #
#    ####
######",
    },
    // Level 142
    VaultWardenLevel {
        data: "\
  ####
  #  #
  #  ####
###$.$  #
#  .@.  #
#  $.$###
####  #
   #  #
   ####",
    },
    // Level 147
    VaultWardenLevel {
        data: "\
       #####
########   #
#.   .  @#.#
#  ###     #
## $  #    #
 # $   #####
 # $#  #
 ## #  #
  #   ##
  #####",
    },
    // Level 63
    VaultWardenLevel {
        data: "\
#####   ####
#   ##### .#
#       $  ########
###  #### .$    @ #
  #  #  #  ####   #
  ####  ####  #####",
    },
    // Level 59
    VaultWardenLevel {
        data: "\
############
#          #
# ####### @##
# #         #
# #  $   #  #
# $$ #####  #
###  # # ...#
  #### #    #
       ######",
    },
    // Level 60
    VaultWardenLevel {
        data: "\
 #########
 #       #
##@##### #
#  #   # #
#  #   $.#
#  ##$##.#
##$##  #.#
#   $  #.#
#   #  ###
########",
    },
    // Level 61
    VaultWardenLevel {
        data: "\
########
#      #
# #### #
# #...@#
# ###$###
# #     #
#  $$ $ #
####   ##
   #.###
   ###",
    },
    // Level 64
    VaultWardenLevel {
        data: "\
 ######
##    #
#   $ #
#  $$ #
### .#####
  ##.# @ #
   #.  $ #
   #. ####
   ####",
    },
    // Level 70
    VaultWardenLevel {
        data: "\
#####
# @ ####
#      #
# $ $$ #
##$##  #
#   ####
# ..  #
##..  #
 ###  #
   ####",
    },
    // Level 75
    VaultWardenLevel {
        data: "\
 #######
## ....##
#   ######
#   $ $ @#
###  $ $ #
  ###    #
    ######",
    },
    // Level 83
    VaultWardenLevel {
        data: "\
######
#    ##
# $ $ ##
## $$  #
 # #   #
 # ## ##
 #  . .#
 # @. .#
 #  ####
 ####",
    },
    // Level 87
    VaultWardenLevel {
        data: "\
     ####
######  #
#       #
#  ... .#
##$######
# $  #
#   $###
##  $  #
 ## @  #
  ######",
    },
    // Level 89
    VaultWardenLevel {
        data: "\
##########
#   ##   #
# $  $@# #
#### # $ #
   #.#  ##
 # #.# $#
 # #.   #
 # #.   #
   ######",
    },
    // Level 90
    VaultWardenLevel {
        data: "\
 ########
 #  @   #
 # $  $ #
### ## ###
#  $..$  #
#   ..   #
##########",
    },
    // Level 118
    VaultWardenLevel {
        data: "\
 #####
##   ##
#  $  ##
# $ $  ##
###$# . ##
  # # .  #
 ## ##.  #
 # @  . ##
 #   #  #
 ########",
    },
    // Level 127
    VaultWardenLevel {
        data: "\
## ####
####  ####
 # $ $.  #
## #  .$ #
#   ##.###
#  $  . #
# @ #   #
#  ######
####",
    },
    // Level 132
    VaultWardenLevel {
        data: "\
####
# @###
#.*  #####
#..#$$ $ #
##       #
 # # ##  #
 #   #####
 #####",
    },
    // Level 66
    VaultWardenLevel {
        data: "\
   ###
   #@#
 ###$###
##  .  ##
#  # #  #
# #   # #
# #   # #
# #   # #
#  # #  #
## $ $ ##
 ##. .##
  #   #
  #   #
  #####",
    },
    // Level 124
    VaultWardenLevel {
        data: "\
##############
#      #     #
# $@$$ # . ..#
## ## ### ## #
 # #       # #
 # #   #   # #
 # ######### #
 #           #
 #############",
    },
    // Level 74
    VaultWardenLevel {
        data: "\
 ####
 #  #######
 #$ @#   .#
## #$$   .#
#  $  ##..#
#   # #####
###   #
  #####",
    },
    // Level 77
    VaultWardenLevel {
        data: "\
##########
# @ .... #
#   ####$##
## #  $ $ #
 # $      #
 #   ######
 #####",
    },
    // Level 80
    VaultWardenLevel {
        data: "\
####
#  #######
#  . ## .#
# $#    .#
## ## # .#
 #    #  #
 #### #  #
  # @$ ###
  # $$ #
  #    #
  ######",
    },
    // Level 91
    VaultWardenLevel {
        data: "\
###########
#    .##  #
# $$@..$$ #
#   ##.   #
###########",
    },
    // Level 102
    VaultWardenLevel {
        data: "\
###########
#....#    #
#  #   $$ #
#  @  ##  #
#     ##$ #
######  $ #
     #    #
     ######",
    },
    // Level 120
    VaultWardenLevel {
        data: "\
      ####
#######  #
# $      ##
# $#####  #
#  @#  #  #
## ##..   #
#  # ..####
# $  ###
# $###
#  #
####",
    },
    // Level 128
    VaultWardenLevel {
        data: "\
  #########
###   #   #
# * $ . . #
#   $ ## ##
####*#   #
 #  @  ###
 #   ###
 #####",
    },
    // Level 131
    VaultWardenLevel {
        data: "\
#####
#   #
# . #
#.@.###
##.#  #
#  $  #
# $   #
##$$  #
 #  ###
 #  #
 ####",
    },
    // Level 135
    VaultWardenLevel {
        data: "\
#######
#     #####
# $$#@##..#
# #       #
#  $ # #  #
#### $  ..#
   ########",
    },
    // Level 92
    VaultWardenLevel {
        data: "\
  ####
  #  #    #####
  #  #    #   #
  #  ######.# #
####  $    .  #
#   $$# ###.# #
#   #   # #   #
######### #@ ##
          #  #
          ####",
    },
    // Level 7
    VaultWardenLevel {
        data: "\
#######
#     #
# .$. #
# $.$ #
# .$. #
# $.$ #
#  @  #
#######",
    },
    // Level 54
    VaultWardenLevel {
        data: "\
      ######
      #    #
  ##### .  #
###  ###.  #
# $  $  . ##
# @$$ # . #
##    #####
 ######",
    },
    // Level 130
    VaultWardenLevel {
        data: "\
#####  #####
#   ####.. #
# $$$      #
#   $#  .. #
### @#  ## #
  #  ##    #
  ##########",
    },
    // Level 134
    VaultWardenLevel {
        data: "\
        ####
#########  #
#   ## $   #
#  $   ##  #
### #. .# ##
  # #. .#$##
  # #   #  #
  # @ $    #
  #  #######
  ####",
    },
    // Level 149
    VaultWardenLevel {
        data: "\
 ####
##  ###
#@$   #
### $ #
 #  ######
 #  $....#
 #  # ####
 ## # #
 # $# #
 #    #
 #  ###
 ####",
    },
    // Level 151
    VaultWardenLevel {
        data: "\
   ####
   #  #
 ###  #
##  $ #
#   # #
# #$$ ######
# #   #   .#
#  $  @   .#
###  ####..#
  ####  ####",
    },
    // Level 35
    VaultWardenLevel {
        data: "\
  ####
 ##  #
 #. $#
 #.$ #
 #.$ #
 #.$ #
 #. $##
 #   @#
 ##   #
  #####",
    },
    // Level 106
    VaultWardenLevel {
        data: "\
 ########
 #      #
 #@   $ #
## ###$ #
# .....###
# $ $ $  #
###### # #
     #   #
     #####",
    },
];

pub const MASTER_LEVELS: &[VaultWardenLevel] = &[
    // Level 116
    VaultWardenLevel {
        data: "\
     #####
   ###   #
####.....#
# @$$$$$ #
#     # ##
#####   #
    #####",
    },
    // Level 133
    VaultWardenLevel {
        data: "\
 #######
 #  . .###
 # . . . #
### #### #
#  @$  $ #
#  $$  $ #
####   ###
   #####",
    },
    // Level 62
    VaultWardenLevel {
        data: "\
   ##########
####    ##  #
#  $$$....$@#
#      ###  #
#   #### ####
#####",
    },
    // Level 125
    VaultWardenLevel {
        data: "\
      #####
      #   ##
      # $  #
######## #@##
# .  # $ $  #
#        $# #
#...#####   #
#####   #####",
    },
    // Level 137
    VaultWardenLevel {
        data: "\
       ####
      ##  ###
####  #  $  #
#  #### $ $ #
#   ..# #$  #
#  #   @  ###
## #..# ###
 # ## # #
 #      #
 ########",
    },
    // Level 78
    VaultWardenLevel {
        data: "\
 #######
##     ##
#  $ $  #
# $ $ $ #
## ### ####
 #@  .....#
 ##     ###
  #######",
    },
    // Level 117
    VaultWardenLevel {
        data: "\
 #### ####
 #  ###  ##
 #      @ #
##..###   #
#      #  #
#...#$  # #
# ## $$ $ #
#  $    ###
####  ###
   ####",
    },
    // Level 121
    VaultWardenLevel {
        data: "\
 ######
 # .  #
##$.# #
#  *  #
# ..###
##$ # #####
## ## #   #
#  #### # #
#   @ $ $ #
##  #     #
 ##########",
    },
    // Level 129
    VaultWardenLevel {
        data: "\
  #########
### @ #   #
# * $ *.. #
#   $ #   #
####*#  ###
 #     ##
 #   ###
 #####",
    },
    // Level 113
    VaultWardenLevel {
        data: "\
   #####
  ##   #
###  # #
#    . #
#  ## #####
#  . . #  ##
#  # @ $   ###
#####. #  $  #
    ####  $  #
       ## $ ##
        #  ##
        #  #
        ####",
    },
    // Level 140
    VaultWardenLevel {
        data: "\
######## #####
#  #   ###   #
#      ## $  #
#.# @ ## $  ##
#.#   # $  ##
#.#    $  ##
#. ## #####
##    #
 ######",
    },
    // Level 108
    VaultWardenLevel {
        data: "\
####     #####
#  ###   #   ##
#    #   #$ $ #
#..# ##### #  #
#  @    # $ $ #
#..#         ##
##   #########
 #####",
    },
    // Level 115
    VaultWardenLevel {
        data: "\
    #####
#####   ####
#     #    #
#  #.....  #
##  ## # ###
 #$$@$$$ #
 #     ###
 #######",
    },
    // Level 122
    VaultWardenLevel {
        data: "\
#####
#   ###
# #$  #
# $   #
# $ $ #
# $#  #
#  @###
## ########
#      ...#
#         #
########..#
       ####",
    },
    // Level 138
    VaultWardenLevel {
        data: "\
  ####
###  #
#    ###
# # . .#
# @ ...####
# # # #   ##
#   # $$   #
#####  $ $ #
    ##$ # ##
     #    #
     ######",
    },
    // Level 141
    VaultWardenLevel {
        data: "\
  ########
  #  # . #
  #   .*.#
  #  # * #
####$##.##
#      $ #
# $ ## $ #
#   @#   #
##########",
    },
    // Level 148
    VaultWardenLevel {
        data: "\
###########
#  .  #   #
# #.  @   #
#  #..# #######
##  ## $$ $ $ #
 ##           #
  #############",
    },
    // Level 152
    VaultWardenLevel {
        data: "\
###### ####
#     #    #
#.##  #$##  #
#   #     #  #
#$  # ###  #  #
# #      #  # #
# # ####  # # #
#. @    $ * . #
###############",
    },
    // Level 95
    VaultWardenLevel {
        data: "\
########
#@     #
# .$$. #
# $..$ #
# $..$ #
# .$$. #
#      #
########",
    },
    // Level 101
    VaultWardenLevel {
        data: "\
  #####
  #   #
  # # #######
  #  *  #   #
  ## ##   # #
  #     #*  #
### # # # ###
#  *#$+   #
# #   ## ##
#   #  *  #
####### # #
      #   #
      #####",
    },
    // Level 109
    VaultWardenLevel {
        data: "\
  #######
# #     #
# # # # #
  # @ $ #
### ### #
#   ### #
# $  ##.#
## $  #.#
 ## $  .#
# ## $#.#
## ## #.#
### #   #
### #####",
    },
    // Level 112
    VaultWardenLevel {
        data: "\
 #####
##   ####
#  $$$  #
# #   $ #
#   $## ##
###  #.  #
  #  #   #
 ##### ###
 #   # ##
 # @....#
 #      #
 #   #  #
 ########",
    },
    // Level 114
    VaultWardenLevel {
        data: "\
######
#    ###
#  # $ #
#  $ @ #
## ## #####
#  #......#
# $ $ $ $ #
##   ######
 #####",
    },
    // Level 97
    VaultWardenLevel {
        data: "\
   ####
   #  ########
#### $ $.....#
#   $   ######
#@### ###
#  $  #
# $ # #
## #  #
 #    #
 ######",
    },
    // Level 111
    VaultWardenLevel {
        data: "\
######
#    ####
#    ...#
#    ...#
######  #
  #  #  #
  # $$ ##
  # @$  #
  # $$  #
  ## $# #
   #    #
   ######",
    },
    // Level 36
    VaultWardenLevel {
        data: "\
####
#  ############
# $ $ $ $ $ @ #
# .....       #
###############",
    },
    // Level 123
    VaultWardenLevel {
        data: "\
########
#      #
# $ $$ ########
##### @##. .  #
    #$  # .   #
    #   #. . ##
    #$# ## # #
    #        #
    #  ###  ##
    #  # ####
    ####",
    },
    // Level 150
    VaultWardenLevel {
        data: "\
     ####
 #####  #
 #     $#######
## ## ..#  ...#
# $ $$#$  @   #
#        ###  #
#######  # ####
      ####",
    },
    // Level 98
    VaultWardenLevel {
        data: "\
#####
#   ## ####
#  $ ### .#
# $   $  .#
## $#####.# ####
# $  # # .###  #
#    # # .#  @ #
###  # #       #
  #### ##     ##
        #######",
    },
    // Level 126
    VaultWardenLevel {
        data: "\
 ###########
##.......  #
# $$$$$$$@ #
#   # # # ##
# # #     #
#   #######
#####",
    },
    // Level 93
    VaultWardenLevel {
        data: "\
 #########
##   #   ##
#    #    #
#  $ # $  #
#   *.*   #
####.@.####
#   *.*   #
#  $ # $  #
#    #    #
##   #   ##
 #########",
    },
    // Level 99
    VaultWardenLevel {
        data: "\
               #####
               #   #
#######  ####### # #
#     #  #  #      #
#  @  ####  #     ####
#  #    ....## ####  #
#    ##### ## $$ $ $ #
######   #           #
         #  ##########
         ####",
    },
    // Level 105
    VaultWardenLevel {
        data: "\
 #### ####
##  ###  ##
#   # #   #
#  *. .*  #
###$   $###
 #   @   #
###$   $###
#  *. .*  #
#   # #   #
##  ###  ##
 #### ####",
    },
    // Level 107
    VaultWardenLevel {
        data: "\
########
#      #
# $*** #
# *  * #
# *  * #
# ***. #
#     @#
########",
    },
    // Level 143
    VaultWardenLevel {
        data: "\
####
#  ####
# $   #
# .#  #
# $# ##
# .  #
#### #
   # #
 ### ###
 #  $  #
## #$# ##
# $ @ $ #
# ..#.. #
###   ###
  #####",
    },
    // Level 139
    VaultWardenLevel {
        data: "\
 ####
##  ####
#   ...#
#   ...#
#   # ##
#   #@ #### ####
##### $   ###  #
    #  ##$ $   #
   ###     $$  #
   # $  ##   ###
   #    ######
   ######",
    },
    // Level 146
    VaultWardenLevel {
        data: "\
 #######
##  .  ##
# .$$$. #
# $. .$ #
#.$ @ $.#
# $. .$ #
# .$$$. #
##  .  ##
 #######",
    },
    // Level 153
    VaultWardenLevel {
        data: "\
#############
#.# @#  #   #
#.#$$   # $ #
#.#  # $#   #
#.# $#  # $##
#.#  # $#  #
#.# $#  # $#
#..  # $   #
#..  #  #  #
############",
    },
    // Level 145
    VaultWardenLevel {
        data: "\
   #####
   # @ #
  ##   ##
###.$$$.###
#  $...$  #
#  $.#.$  #
#  $...$  #
###.$$$.###
  ##   ##
   #   #
   #####",
    },
    // Level 144
    VaultWardenLevel {
        data: "\
   ####
 ###  #####
 # $$ #   #
 # $ . .$$##
 # .. #. $ #
### #** .  #
#  . **# ###
# $ .# .. #
##$$.@. $ #
 #   # $$ #
 #####  ###
     ####",
    },
    // Level 155
    VaultWardenLevel {
        data: "\
    ######               ####
#####*#  #################  ##
#   ###                      #
#        ########  ####  ##  #
### ####     #  ####  ####  ##
#*# # .# # # #     #     #   #
#*# #  #     # ##  # ##  ##  #
###    ### ###  # ##  # ##  ##
 #   # #*#      #     # #    #
 #   # ###  #####  #### #    #
 #####   #####  ####### ######
 #   # # #**#               #
## # #   #**#  #######  ##  #
#    #########  #    ##### ###
# #             # $        #*#
#   #########  ### @#####  #*#
#####       #### ####   ######",
    },
];
