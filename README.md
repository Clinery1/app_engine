# About
This is a relatively simple crate that acts as kind of a framework for my future GUI applications so
I
- Don't have to rewrite a renderer/UI/filepicker/config loader/etc. every time.
- Allows updates to a common renderer when I want to update it or rewrite it to use bare Vulkan
    instead of the `screen-13` crate.
- Has a "good enough" renderer for UI and simple apps, but allows for more custom processes if
    needed by having the entire renderer open and public.
- Allow very easy hacking of the UI system
- Make a "better" UI system than what I have used before (see [the UI explanation](#ui-explanation)
    for more info)

# UI explanation
(NOTE: At the time of writing, the UI is not finished, but this outlines the goal)
The UI in this crate is designed to be very simple, and have some things in common with HTML layout
logic.

The entire system is based on the concept of containers and displays.
Containers don't show anything useful, but do have a background, borders, and a layout. They also
handler user input, so things like buttons are done with containers. Containers can either be docked
to an edge, or floating. In the future I want to support dragging a container to another window, but
that is a long ways off.
Displays are the container's counterpart. They do stuff like show images, display text, draw
symbols, etc. Where a container only contains other elements, displays only show things.

I will implement a class formatting system. It will allow for common layout settings like borders,
colors, etc. to be specified in a "class" like in CSS and applied to an element just like in CSS
with a name.

There will also be a separate caching system for on-the-fly generated images (think rounded borders
for a specific element that can be reused each frame) and font data.

## Layout
When using other UI crates (egui and iced) I found the layout to be... A bit lacking. I probably
didn't learn enough about them, but coming from building web apps, their layout options didn't seem
good. So I have decided to write my own immediate layout system and integrate into my own renderer.

This new layout system will follow some examples set by browser's CSS layout. There will be a layout
container that can layout vertically, horizontally, or distribute in a grid (these might be distinct
types, but still containers).

Buttons are just a clickable container, so adding text is as easy as adding a text widget.
