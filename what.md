# What ?

This is an ongoing file recapping my interrogations as I go through them.

- Is cargo chef really useful ?
  - why does dockerfile with chef had an error that cargo wasn't found ?
- Do I need all external dependencies in both compile time and runtime ? (apt installs)
- do NOT copy . . in dockerfile because the cache  is invalidated at each minor change
- 