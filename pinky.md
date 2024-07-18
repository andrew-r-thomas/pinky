---
title: pinky
---

ok so this i am thinking about making a sort of "second brain" system.
part of this is to have a nice markdown experience in nvim that is
obsidian-like. but also, i want to have this be much more deeply integrated
into the entire file system, and maybe the os. for example, i dont want to
confine notes to one central folder, i want them to be able to be wherever
in the file system i want, so like i can make a markdown file in a code
project repo that is my "project" note for that project. and i want to have
things like TODOs that i write in the source files to show up in my project
note.

i also want to be able to sort of "link from anywhere" so any text file can
have a link to any file in the system, and for other text files we can link to
specific lines, and for data files we can make some embed systems to show them
in the notes (like images), we could also do embeds for text stuff, although
i would personally not use that very much

also, we want really good todo management, probably the ability to make your
own statuses, etc

also we want just nice editing, so like keyboard shortcuts for moving shit
around and whatnot.

we could maybe do this all from a langauge server, and just have things like
keyboard shortcuts map to sending messages to the server, im not super sure.

alternatively, we make this its own system, and we could make plugins for
for different editors, and maybe also have something like a cli, idk

one question is how we will keep everything in sync if we are so intertwined
with the file system, like if we move some files, we're gonna have to update
some stuff about like where the links are pointing to and stuff, and we wont
always be doing stuff from an editor like neovim, we might just put an image
in a different spot, so yeah idk man

it seems like we can listen to file system changes, but would this mean we need
a process running all the time, or is there a way to have the os start a
process whenever there is a file system change, i feel like there should be.
