# TODO

- [x] fix tentative type error message
- [x] ~consider using lazylock(refcell(fxhashmap)) for the symbol table~
- [x] ~use rpds (persistent data structure crate) for speed?~
- [x] use write! macro for code emission
- [x] ~simplify parser types with a macro?~
- [x] ~type table should have type HashMap<&Exp, &Symbol>?~
- [x] post ++ as ((A+=1)-1), and
- [ ] compound assignment as primitive binary operation tacky node?
- [ ] make preprocesser
- [ ] type check switch condition
- [ ] leaked &str instead of string for clone heavy operations, especially in asm_gen
- [ ] separate function type apart from others to implement Copy for Type
- [ ] cleaner shift handling in codegen
