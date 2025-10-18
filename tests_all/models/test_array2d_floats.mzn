% Test array2d with float values (from Hakank arbitrage example pattern)
int: n = 2;
int: m = 2;

array[1..n, 1..m] of float: rates = array2d(1..n, 1..m,
  [0.0, 1.5,
   0.6, 0.0]);

output [
  "rates[1,1] = ", show(rates[1,1]), "\n",
  "rates[1,2] = ", show(rates[1,2]), "\n",
  "rates[2,1] = ", show(rates[2,1]), "\n",
];
