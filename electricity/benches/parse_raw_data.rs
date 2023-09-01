use criterion::{black_box, criterion_group, criterion_main, Criterion};
use electricity::{parse_raw_data_to_data, ElectricityFailuresRawData};

pub fn benchmark(c: &mut Criterion) {
    let data = ElectricityFailuresRawData {
        id: String::from("id"),
        date: String::from("01-01-2021"),
        url: String::from("url"),
        html: String::from(
            r#"
            <html>
                <body>
                    <table>
                        <tbody>
                            <tr>
                                <td>
                                    <b>Скопје - Центар - 01.01.2021</b>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                    <table>
                        <tbody>
                            <tr>
                                <td>Општина</td>
                                <td>Време</td>
                                <td>Улице</td>
                            </tr>
                            <tr>
                                <td>Центар</td>
                                <td>08:00 - 16:00</td>
                                <td>Бул. Климент Охридски: 43-46</td>
                            </tr>
                        </tbody>
                    </table>
                </body>
            </html>
        "#,
        ),
        hash: String::from("hash"),
        version: 1,
    };

    c.bench_function("parse_data", |b| {
        b.iter(|| parse_raw_data_to_data(black_box(&data)))
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
