# 📈 Minha Carteira - Gerenciador de Investimentos

Uma aplicação web fullstack de alta performance desenvolvida em **Rust** para gerenciamento e acompanhamento de carteira de investimentos em tempo real.

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-000000?style=for-the-badge&logo=rust&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)

---

## 🚀 Funcionalidades

- **Autenticação Segura:** Cadastro e login de usuários com criptografia de senha (`bcrypt`) e gerenciamento de sessões via cookies/JWT.
- **Gestão de Ativos:** Cadastro e listagem de investimentos (Ações BR, FIIs, Ações EUA, Criptomoedas e Renda Fixa).
- **Cálculo Automático:** Cálculo dinâmico do total investido com base na quantidade e preço médio informado.
- **Interface Moderna:** Dashboard em Dark Mode responsivo estilizado com Tailwind CSS.

---

## 🛠️ Tecnologias Utilizadas

### Backend
- **[Rust](https://www.rust-lang.org/):** Linguagem principal do projeto.
- **[Axum](https://github.com/tokio-rs/axum):** Framework web modular baseado em Tokio.
- **[SQLx](https://github.com/launchbadge/sqlx):** Driver assíncrono para PostgreSQL com checagem de queries em tempo de compilação.
- **[Askama](https://github.com/djc/askama):** Engine de templates HTML compilada em Rust.

### Banco de Dados
- **PostgreSQL:** Executado via Docker para armazenamento contínuo de usuários e ativos.

### Frontend
- **HTML5 & Tailwind CSS:** Estilização utilitária para Dark Mode e navegação fluida.

---

## ⚙️ Como Rodar o Projeto Localmente

### Pré-requisitos
- [Rust & Cargo](https://www.rust-lang.org/tools/install) instalados.
- [Docker](https://www.docker.com/) rodando na máquina.
