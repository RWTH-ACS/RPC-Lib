#include "math.h"

int *add_1_svc(int arg1, int arg2,  struct svc_req *rqstp) {
    static int result;
    result = arg1 + arg2;
    return &result;
}
