module Impl.Test exposing (hello, hello_from_import)

import Impl.Test.Other

hello =
    "Hello from Impl.Test"

hello_from_import =
    Impl.Test.Other.hello

hello_from_prelude =
    String.append "Hello, " "from prelude"
