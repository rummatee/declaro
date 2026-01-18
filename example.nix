{
  variable2 ? "defaultValue",
  variable1,
  moreVariables,
  ...
}: {
  someAttribute = "This is an example attribute";
  nested = {
    attribute = "value";
  };
  a.b.c = 42;
  listExample = [1 2 3 4 5];
  more = variable2;
  combined = "${variable1}-${variable2}";
}
