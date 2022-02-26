#include "test.h"

int *add_1_svc(int arg1, int arg2, struct svc_req *rqstp) {
    static int result;
    result = arg1 + arg2;
    return &result;
}

int *struct_mul_fields_1_svc(MyStruct arg1, struct svc_req *rqstp) {
    static int result;
    result = arg1.x * arg1.y;
    return &result;
}

MyStruct *struct_combine_1_svc(int arg1, int arg2, struct svc_req *rqstp) {
    static MyStruct result;
    result.x = arg1;
    result.y = arg2;
    return &result;
}

ResultUnion *union_test_1_svc(int arg1, struct svc_req *rqstp) {
    static ResultUnion result;
    switch(arg1) {
        case 0:
            result.err = 0;
            result.ResultUnion_u.int_res = 1;
            break;
        case 20:
            result.err = 20;
            result.ResultUnion_u.float_res = 1.0f;
            break;
        default:
            result.err = -1;
            result.ResultUnion_u.int_res = 0;
            break;
    }
    return &result;
}

int *union_param_1_svc(ResultUnion arg1, struct svc_req *rqstp) {
    static int result;
    switch(arg1.err) {
        case 0:
            result = arg1.ResultUnion_u.int_res;
            break;
        case 20:
            result = (int) arg1.ResultUnion_u.float_res;
            break;
        default:
            result = -1;
            break;
    }
    return &result;
}
