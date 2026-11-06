use std::io::{self, Write};

const NETWORK_FEE: f64 = 0.000_005;

#[derive(Clone, Copy, PartialEq)]
enum Network {
    Devnet,
    MainnetBeta,
}

impl Network {
    fn label(self) -> &'static str {
        match self {
            Network::Devnet => "Solana Devnet (учебная сеть)",
            Network::MainnetBeta => "Solana Mainnet Beta (основная сеть)",
        }
    }
}

struct Transaction {
    recipient: String,
    amount: f64,
    fee: f64,
    network: Network,
}

struct Wallet {
    owner: String,
    balance: f64,
    network: Network,
    history: Vec<Transaction>,
}

impl Wallet {
    fn show_info(&self) {
        println!("\n=== Solana Wallet CLI ===");
        println!("Владелец: {}", self.owner);
        println!("Сеть: {}", self.network.label());
        println!("Баланс: {:.6} SOL", self.balance);
        println!("Операций в истории: {}", self.history.len());
    }

    fn transfer(&mut self, recipient: String, amount: f64) -> Result<(), String> {
        if self.network != Network::Devnet {
            return Err(
                "Переводы в симуляторе разрешены только в Devnet. Переключитесь на Devnet."
                    .to_string(),
            );
        }

        if recipient.len() < 32 || recipient.len() > 44 {
            return Err("Адрес получателя должен содержать от 32 до 44 символов.".to_string());
        }

        if !amount.is_finite() || amount <= 0.0 {
            return Err("Сумма перевода должна быть положительным числом.".to_string());
        }

        let total = amount + NETWORK_FEE;
        if total > self.balance {
            return Err(format!(
                "Недостаточно средств. Нужно {:.6} SOL с учётом комиссии.",
                total
            ));
        }

        self.balance -= total;
        self.history.push(Transaction {
            recipient,
            amount,
            fee: NETWORK_FEE,
            network: self.network,
        });

        Ok(())
    }

    fn show_history(&self) {
        println!("\n=== История операций ===");

        if self.history.is_empty() {
            println!("Операций пока нет.");
            return;
        }

        for (index, transaction) in self.history.iter().enumerate() {
            println!(
                "{}. Перевод {:.6} SOL на {} | комиссия {:.6} SOL | {}",
                index + 1,
                transaction.amount,
                transaction.recipient,
                transaction.fee,
                transaction.network.label()
            );
        }
    }
}

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn choose_network() -> io::Result<Network> {
    loop {
        println!("\nВыберите сеть:");
        println!("1 — Solana Devnet");
        println!("2 — Solana Mainnet Beta");

        match read_line("Ваш выбор: ")?.as_str() {
            "1" => return Ok(Network::Devnet),
            "2" => return Ok(Network::MainnetBeta),
            _ => println!("Введите 1 или 2."),
        }
    }
}

fn run() -> io::Result<()> {
    let mut wallet = Wallet {
        owner: String::from("SolaraDev26"),
        balance: 10.0,
        network: Network::Devnet,
        history: Vec::new(),
    };

    println!("Учебный симулятор кошелька Solana.");
    println!("Программа не подключается к блокчейну и не использует реальные средства.");

    loop {
        println!("\n--- Меню ---");
        println!("1 — Показать кошелёк");
        println!("2 — Выполнить тестовый перевод");
        println!("3 — Показать историю операций");
        println!("4 — Сменить сеть");
        println!("0 — Выйти");

        match read_line("Выберите действие: ")?.as_str() {
            "1" => wallet.show_info(),
            "2" => {
                let recipient = read_line("Введите Devnet-адрес получателя: ")?;
                let amount_input = read_line("Введите сумму в SOL: ")?;

                match amount_input.replace(',', ".").parse::<f64>() {
                    Ok(amount) => match wallet.transfer(recipient, amount) {
                        Ok(()) => println!("Тестовый перевод успешно добавлен в историю."),
                        Err(message) => println!("Ошибка: {message}"),
                    },
                    Err(_) => println!("Ошибка: введите сумму числом, например 0.5."),
                }
            }
            "3" => wallet.show_history(),
            "4" => {
                wallet.network = choose_network()?;
                println!("Выбрана сеть: {}", wallet.network.label());
            }
            "0" => {
                println!("Работа программы завершена.");
                break;
            }
            _ => println!("Неизвестная команда. Выберите пункт из меню."),
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Ошибка ввода-вывода: {error}");
    }
}
