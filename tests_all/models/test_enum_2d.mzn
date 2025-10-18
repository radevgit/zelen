% Test enum 2D array
enum Color = {Red, Green, Blue};

array[1..2, 1..3] of var Color: colors;

solve satisfy;
