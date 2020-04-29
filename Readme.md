# Guild Wars Skill Cards

![Rust](https://github.com/regendo/guild-wars-skill-cards/workflows/Rust/badge.svg)

A tool to create playing cards for all ~1300 Guild Wars skills. Create your cards, import them into Tabletop Simulator, and draft your team build out of random draws!

## Usage

1. [Install rust.](https://www.rust-lang.org/tools/install)
2. Clone this repository.

```bash
git clone git@github.com:regendo/guild-wars-skill-cards.git
cd guild-wars-skill-cards
```

3. Compile and run the program. This may take a while—we're downloading a lot of images from the wiki and we wait between downloads to be nice. We're also doing a lot of image manipulation. However, if you need to cancel the program execution at any time, you can re-run this command to resume where you left off.

```bash
cargo run
```

4. Use the images in the `cards/decks` directory to create custom decks in Tabletop Simulator.

## Drafting

Use Tabletop Simulator's "cut" to take the desired amount of cards out of a deck, then use "split" to split that stack evenly into smaller, booster-sized stacks.

In card games like Magic the Gathering, drafting works as follows:

1. Each player opens one booster pack.
2. Each player takes ("drafts") one card from their pack, then passes the rest to the person on the left.
3. Repeat from 2. until there are no cards left in the packs.
4. Repeat from 1. until you have opened as many packs as you initially agreed on.
5. Each player builds a deck out of the drafted cards, then plays with that deck.

We can easily adapt that format to skill cards. If you draft alone, decide on a number of fake players and give each of them a booster pack (face-down). Then, whenever a fake player would draft a card, instead shuffle the face-down booster and discard one card from it. Instead of building a deck, create a team build for you and your heroes out of the drafted cards.

Use each card only once: only give Energy Surge to two of your heroes if you've actually drawn it twice. (You need to duplicate each deck if you want to draw multiple copies of cards.)

For safety reasons, remove Resurrection Signet from the card pool before drafting. Add a free Resurrection Signet to any character or hero build whenever you want.

Personally I've had a lot of fun drafting on a new character with four fake players and 10 cards per pack. I've drafted one round of one pack each when I started the game, and another round after each campaign mission I've completed.

That said, what you do with your cards is up to you. Play them however you find most fun! Create a new character under your own challenge rules or draft a lot of packs at once to fill out your exiting character's team build.

## Licenses

This program is dual-licensed under the MIT and Apache 2.0 licenses, as is recommended for rust programs.

Please note that many of the assets used to create these cards were created by other people. They are re-distributed here in accordance with their respective licenses. This program's licenses do not apply to them.
