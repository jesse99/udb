#include <stdio.h>

#include "item.h"

int main() {
    int cost = 0;

    add_to_cart(1);
    add_to_cart(1);
    add_to_cart(1);
    add_to_cart(1);

    add_to_cart(2);
    add_to_cart(2);
    add_to_cart(2);

    while (cost < 1000) {
        cost += remove_from_cart(2) * get_cost(2);
    }

    printf("%s cost: %d\n", get_name(1), cost);
    return 0;
}










