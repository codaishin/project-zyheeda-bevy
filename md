```
los(a, b):
  - a is b -> true
	- b is neighbor of a -> true
	- closest 2 neighbors towards b:
	  - both are not neighbors of each other -> false
		- otherwise -> los(furthest neighbor, b)
	- less that 2 neighbors -> false
```
