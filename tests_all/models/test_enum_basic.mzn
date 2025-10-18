% Test basic enum functionality
enum Color = {Red, Green, Blue};

var Color: my_color;

constraint my_color != Red;

solve satisfy;
