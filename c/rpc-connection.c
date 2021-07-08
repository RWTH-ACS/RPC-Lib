#include "rpc-connection.h"

void rpc_deinitialize(CLIENT *client) {
    clnt_destroy(client);
}
