- think about one more enemy. maybe fixed one that is fixed but can rotate and is rotating & expanding.
  - more enemies ideas:
    - The Mimic - Copies the player's drawing behavior by creating its own lines that can box the player in if they're not careful.
    - The Chaser - Once a player starts drawing, gradually chases.
    - The Splitter - When hit or after a set time, splits into two smaller, faster versions of itself.
    - The Shooter - fixed position but shoots small bullets that travel slowly towards the player path
- think about the criteria for procedurally spawning enemies.
  - only based on seed and level, not score or %
- New Enemy: Square: use easing BounceInOut to animate alpha &
  - moves teleporting to places with no shapes or templinesegments.
    - to teleport: query for shapes, and spawn in coordinates
      generated at random that are within bounds & don't collide
      with walls or with points in shape.
      don't care about templinesegmnet, if we teleport on a
      drawing taht is in progress the we reward the player.
      coords of walls can be obtained without a query.
- implement the basic for seeding the game. that is:
  - most enemies should have their spawn factor include some random
    variables that, if we first generate a seed (few easy & chars)
    we can then display that seed in the game over screen so a good
    run can be replayed.
- substitute colors for balatro palette.
- when eating an enemy, the enemy eaten should die in a way that is visible to the player, some animation maybe?
- items: point multipliers, stop time, maybe something that splits the line and it automatically tries to reach the wall.
- different enemies
- animations for next level and gave over.
- improve ui on displayed level/points/progress
- add README, cite inspirations.

- menu ui.
- game over screen.
