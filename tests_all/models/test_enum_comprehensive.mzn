% Comprehensive enum test
enum Team = {Red, Blue, Yellow, Green};
enum Player = {Alice, Bob, Charlie, David};

int: num_players = 4;

var Team: alice_team;
var Team: bob_team;
array[1..num_players] of var Team: all_teams;

constraint alice_team != bob_team;
constraint alldifferent(all_teams);

solve satisfy;
