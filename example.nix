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
  a.b.c = "deepValue";
  reference = variable2;
}
