<h1> Payments Engine </h1> <br />
- process payments for set of input transactions. <br />

To Run the code <br />
cargo build <br />
cargo run -- ${csv file} > account.csv <br />


Input file should be comma separted. Whitespaces are accepted by the application. 

- I have used Serde deserialize for the correctness of the input file.
- I have run testcases with multiple invalid scenarios and the application was able to handle it. I am attaching a few invalid scenarios I have tested it aganist(In the email).

Assumptions Made
- If a dispute is in progress for a transaction and either a chargeback or resolve was completed then for a new resolve or chargeback another dispute should be sent with the same transaction ID.

- Regarding the available, held and total balance changes I have followed the question.


