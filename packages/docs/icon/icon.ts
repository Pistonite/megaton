

const shaft_color = "#5E2419"; // reddish
//const shaft_color = "#8C471C"; // orangeish
const gear_color = "gray";
const shell_color = "";//"#ff889988";
const shell_color2 = "";//"#5E241988";
const thickness = 2;

function make_gear(x: number) {
  const corner = size(thickness, thickness, thickness)
    .at(x, 2, 2)
  return size(thickness, 12, 12)
    .at(x, 2, 2)
    .difference(corner)
    .difference(corner.translated(0, 0, 10))
    .difference(corner.translated(0, 10, 10))
    .difference(corner.translated(0, 10, 0));
}

function make_big_gear(x: number) {
  const base = size(thickness, 12, 12)
    .at(x, 2, 2);
  const s1 = base.dextended("y", -5)
  .dextended("z", 2);
  const s2 = base.dextended("z", -5)
  .dextended("y", 2);
  const s3 = base.dextended("y", -5)
   .dextended("z", -5)
   .dextended("x", 5);
  return base.union(s1).union(s2).union(s3);
}

function render_inside() {
  make_gear(11).render(gear_color);
  make_gear(3).render(gear_color);
  make_big_gear(7).render(shaft_color);
}

render_inside()
if (shell_color2) {
  size(14,14,14).at(1,1,1).render(shell_color2);
}
if (shell_color) {
  size(14,14,14).at(1,1,1).dextended("x", -4).render(shell_color);
}