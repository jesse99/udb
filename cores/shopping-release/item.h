#ifndef ITEM_H
#define ITEM_H

void add_to_cart(long id);
int remove_from_cart(long id);

int get_cost(long id);
const char *get_name(long id);

#endif  // ITEM_H
