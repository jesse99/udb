#include "item.h"

#include "list.h"

#include <stdio.h>
#include <stdlib.h>

struct item {
    long id;
    int count;
    LIST_ENTRY(item) next;
};

// TODO use TLS
static LIST_HEAD(shopping_head, item) *shopping_cart;

void add_to_cart(long id)
{
    struct item *item;

    LIST_FOREACH(item, shopping_cart, next) {
        if (item->id == id) {
            item->count += 1;
            return;
        }
    }

    item = malloc(sizeof(struct item));
    item->id = id;
    item->count = 1;
    LIST_INSERT_TAIL(shopping_cart, item, item, next);
}

int remove_from_cart(long id)
{
    struct item *item;
    int count;

    LIST_FOREACH(item, shopping_cart, next) {   // ok because we bail right after the remove
        if (item->id == id) {
            count = item->count;
            LIST_REMOVE(item, next);
            free(item);
            return count;
        }
    }
    printf("failed to find %ld in shopping cart\n", id);

    // this will core
    LIST_REMOVE(item, next);
    free(item);

    return 0;
}

int get_cost(long id)
{
    if (id == 1) {
        return 10;
    } else if (id == 2) {
        return 12;
    } else {
        return 0;
    }
}

const char *get_name(long id)
{
    if (id == 1) {
        return "apple";
    } else if (id == 2) {
        return "banana";
    } else {
        return "bad id";
    }
}

