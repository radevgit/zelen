% Test model with parameters that will be provided by data file
int: n;  % Parameter to be set in data file
array[1..n] of int: costs;  % Parameter array to be set in data file

var 1..n: choice;
constraint costs[choice] >= 15;
solve minimize costs[choice];
