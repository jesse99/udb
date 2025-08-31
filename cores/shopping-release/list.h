#ifndef LIST_H
#define LIST_H

#define LIST_HEAD(name, type)                                           \
struct name {                                                           \
    struct type *lh_first;      /* first element */                     \
}

#define LIST_ENTRY(type)                                                \
struct {                                                                \
    struct type *le_next;       /* next element */                      \
    struct type **le_prev;      /* address of previous next element */  \
}

#define LIST_FIRST(head)            ((head)->lh_first)
#define LIST_NEXT(elm, field)       ((elm)->field.le_next)

#define LIST_INSERT_AFTER(listelm, elm, field) do {                     \
    if (((elm)->field.le_next = (listelm)->field.le_next) != NULL)      \
        (listelm)->field.le_next->field.le_prev = &(elm)->field.le_next;\
    (listelm)->field.le_next = (elm);                                   \
    (elm)->field.le_prev = &(listelm)->field.le_next;                   \
} while (/*CONSTCOND*/0)

#define LIST_INSERT_HEAD(head, elm, field) do {                         \
    if (((elm)->field.le_next = (head)->lh_first) != NULL)              \
        (head)->lh_first->field.le_prev = &(elm)->field.le_next;        \
    (head)->lh_first = (elm);                                           \
    (elm)->field.le_prev = &(head)->lh_first;                           \
} while (/*CONSTCOND*/0)

#define LIST_INSERT_TAIL(head, elm, type, field) do {                   \
	struct type *curelm = LIST_FIRST(head);                             \
	if (curelm == NULL) {                                               \
		LIST_INSERT_HEAD(head, elm, field);                             \
	} else {                                                            \
		while (LIST_NEXT(curelm, field))                                \
			curelm = LIST_NEXT(curelm, field);                          \
		LIST_INSERT_AFTER(curelm, elm, field);                          \
	}                                                                   \
} while (/*CONSTCOND*/0)

#define LIST_REMOVE(elm, field) do {                                    \
    if ((elm)->field.le_next != NULL)                                   \
        (elm)->field.le_next->field.le_prev = (elm)->field.le_prev;     \
    if ((elm)->field.le_prev != NULL)                                   \
        *(elm)->field.le_prev = (elm)->field.le_next;                   \
} while (/*CONSTCOND*/0)

#define LIST_FOREACH(var, head, field)                                  \
    for ((var) = ((head)->lh_first);                                    \
         (var);                                                         \
         (var) = ((var)->field.le_next))
         
#endif  // LIST_H

