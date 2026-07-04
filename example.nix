{
  variable2 ? "defaultValue",
  variable1,
  moreVariables,
  ...
}: {
  someAttribute = "This is an example attribute ${variable1}";
  nested = {
    attribute = "value";
  };
  a.b.c = "deepValue";
  reference = variable2;
  arithmetic = 1 + 3;
}
