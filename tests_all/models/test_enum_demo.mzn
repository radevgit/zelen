% Simplified version of bobsledders puzzle to test enum support
% This demonstrates the new enumerated type feature in Zelen

enum Names = {Alice, Bob, Charlie};
enum Colors = {Red, Blue, Green};

int: num_people = 3;

% Decision variables - each person has a color
array[1..num_people] of var Colors: person_colors;

% Simple constraints
constraint alldifferent(person_colors);

solve satisfy;
