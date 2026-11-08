---
title: Difficulty
nav_order: 4.35
---

# Difficulty

Each character plays on one of four difficulties. A server may allow only some of them. If yours stops allowing the one you play on, you are moved to the nearest it does allow (the easier one if two are as near).

| Difficulty | Harm you take | Health and Mana refill | When you die |
| ---------- | ------------- | ---------------------- | ------------ |
| Easy | A fifth less | Twice as fast | Nothing is lost. |
| Normal | As usual | As usual | As the game is meant to be played. |
| Hard | A fifth more | As usual | You drop everything but what is worn against the skin. |
| Ironman | A fifth more | As usual | You drop everything, and the character is gone for good. |

## Choosing

When you make a character, choose their difficulty on the character screen, from those the server allows. It starts on your own difficulty, which you set in **Settings**, **Gameplay**, before you play.

While playing, you can change between Easy, Normal and Hard at any time in **Settings**, **Gameplay**, from those the server allows. Ironman is only chosen when making a character, and you are asked first, as death is then permanent. An Ironman character can never change to another difficulty, and no character can change to Ironman.

## Dying

On Normal and Easy you keep everything when you die. You wake on the world map with full Health, outside the place you died.

On Hard, everything you carry and wear falls in a bag where you died, except clothes worn against the skin alone, such as a T-shirt or socks. Rings, necklaces, weapons and anything worn over other things drop. Go back for the bag before someone else finds it.

On Ironman, everything you carry and wear falls where you died, and the character is deleted. You are taken back to making a new character.

## For server hosts

A server allows every difficulty unless `HOW_DIFFICULTIES` lists the ones it allows: see [Environment variables](environment-variables). When you play on the game's own server, tick the difficulties it allows in **Server Options** (the gear beside **Online**). At least one stays ticked.
