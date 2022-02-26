# Parameters
SERVER = test_server
SOURCES.x = test.x
TARGETS_SVC.c = test_svc.c test_server.c test_xdr.c 
TARGETS = test.h test_xdr.c test_svc.c test_server.c
OBJECTS_SVC = $(TARGETS_SVC.c:%.c=%.o)

# Compiler flags 
CFLAGS += -g -I /usr/include/tirpc
LDLIBS += -ltirpc
RPCGENFLAGS = -N

# Targets 
all : $(CLIENT) $(SERVER)
$(TARGETS) : $(SOURCES.x) 
	rpcgen $(RPCGENFLAGS) $(SOURCES.x)
$(OBJECTS_SVC) : $(TARGETS_SVC.c) 
$(SERVER) : $(OBJECTS_SVC) 
	$(LINK.c) -o $(SERVER) $(OBJECTS_SVC) $(LDLIBS)
clean:
	 $(RM) core $(TARGETS) $(OBJECTS_CLNT) $(OBJECTS_SVC) $(SERVER)
