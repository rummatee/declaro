{
  variable2 ? "defaultValue",
  variable1,
  moreVariables,
  ...
}: {
  someAttribute = "This is an example attribute ${variable1}";
  nested = let
    inner = "innerValue";
  in {
    attribute = "value";
    innerAttribute = inner;
  };
  a.b.c = "deepValue";
  reference = variable2;
  arithmetic = 1 + 3;
}
