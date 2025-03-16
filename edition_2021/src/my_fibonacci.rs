pub fn fibonacci(n_fibonacci: u8) -> usize {
    // Fibonacci Variable initalisieren
    let mut fibonacci: usize = 0;

    match n_fibonacci {
        0 => (), // Keine Änderung, fibonacci = 0
        1 | 2 => fibonacci = 1,
        _ => {
            // Fibonacci Hilfs-Variablen initalisieren
            let mut fibonacci_m1 = 1; //f_n-1
            let mut fibonacci_m2 = 1; //f_n-2

            // Fibonacci-Zahl aus den Vorgängern berechnen: f_n = f_n-1 + f_n-2
            for _n in 3..=n_fibonacci {
                fibonacci = fibonacci_m1 + fibonacci_m2;
                // f_n-1 und f_n-2 "shiften"
                fibonacci_m2 = fibonacci_m1;
                fibonacci_m1 = fibonacci;
            }
        }
    }
    fibonacci
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fibonacci() {
        // Übung: Gebe die n-te Zahl der Fibonacci-Folge aus.
        // Fibonacci-Folge in moderner Schreibweise beginnend mit einer 0
        // Die 0-te Zahl der Fibonacci-Folge ist 0, die 1-te 1, die 2-te 1, etc.

        // Festlegen der gesuchten Fibonacci-Zahl
        // Mit u64 lässt sich maximal die 93 Fibonacci-Zahl berechnen
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(6), 8);
        assert_eq!(fibonacci(8), 21);
    }
}
