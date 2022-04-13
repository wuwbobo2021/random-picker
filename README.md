# random-picker
Generate random choices such as activities, places, foods that have different costs or prizes, based on a distribution table of relative values of probability.
```
g++ table.cpp picker.cpp main.cpp -O3 -o random-picker -march=native
```
## Known Problem
When picking more than one items from the table with the repetitive mode turned off, the probability of each item will be higher and can be calculated by:
```
Pi_1 = pi / s;

Pi_2 = Pi_1 + Σ(j=1~n ex i)(pj/s)pi/(s-pj);

Pi_3 = Pi_2 + Σ(j=1~n ex i)Σ(k=1~n ex i,j)[pj/s * pk/(s-pj) * pi/(s-pj-pk)];

Pi_4 = Pi_3 + Σ(i1=1~n ex i)Σ(i2=1~n ex i,i1)Σ(i3=1~n ex i,i1,i2)
              [pi1/s * pi2/(s - pi1) * pi3/(s - pi1 - pi2) * pi/(s - pi1 - pi2 - pi3)];

Pi_m = Pi_<m-1> + pi * Σ(i1=1~n ex i)Σ(i2=1~n ex i,i1)...Σ(i<m-1>=1~n ex i,i1...i<m-2>)
                       {Π(j=1~m-1)[pij/(s-Σ(j'=1~j-1)pij')] / (s - Σ(j=1~m-1)pij)}
(1 < m < n);

Pi_n = 1/n.
```
in which the values in the table are p1 p2 p3...pn, 1 ≤ i ≤ n; s = Σ(i=1...n)pi; Pi_m is the probability of <i>th item's occurence when picking m items; i1...i<m-1> represent different levels of iterators.

I am unable to simplify the formula above, that involves higher mathematics.
