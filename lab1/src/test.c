#include <stdio.h>
#include <stdbool.h>

// Объявление функции
int sum(int a, int b) {
    return a + b;
}

int main() {

    /*тут я решил написать двустрочный комментарий*/
    /*
    двустрочный-двустрочный комментарий*/

    // переменные объявляем 
    int x = 10;
    int y = 5;
    int result = 0;

    bool flag = true;

    // арифметическое выражение
    result = x * y + (x - y) / 5;

    // Логическое выражение + if-else
    if (result > 20 && flag) {
        printf("Result is large\n");
    } else {
        printf("Result is small\n");
    }

    for (int i = 0; i < 5; i++) {
        result += i;
    }

    int j = 0;
    while (j < 3) {
        result -= j;
        j++;
    }

    int total = sum(result, 100);

    printf("Total = %d\n", total);

    return 0;
}
