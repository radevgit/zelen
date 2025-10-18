% Test enum array
enum Color = {Red, Green, Blue};

array[1..3] of var Color: colors;

constraint alldifferent(colors);

solve satisfy;
