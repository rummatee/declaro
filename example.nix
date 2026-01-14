{
  variable1,
  variable2 ? "defaultValue",
  ...
}: {
  someAttribute = "This is an example attribute";
  nested = {
    attribute = "value";
  };
  a.b.c = 42;
  listExample = [1 2 3 4 5];
  more = variable1;
  combined = "${variable1}-${variable2}";
}

