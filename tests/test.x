struct MyStruct {
    int x;
    int y;
};

union ResultUnion switch(int err) {
    case 0:
        int int_res;
    case 20:
        float float_res;
    default:
        void;
};

program TEST {
    version VERS {
        int         ADD(int, int)                = 1;
        int         STRUCT_MUL_FIELDS(MyStruct)  = 2;
        MyStruct    STRUCT_COMBINE(int, int)     = 3;
        ResultUnion UNION_TEST(int)              = 4;
        int         UNION_PARAM(ResultUnion)     = 5;
    } = 1;
} = 500000;
