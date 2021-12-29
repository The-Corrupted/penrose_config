use penrose::core::{
    client::Client,
    data_types::{Region, ResizeAction},
    xconnection::Xid
};


/* Layout visualization
 * -------------  -------------  -------------
 * |           |  |           |  |           |
 * |           |  |           |  |           |
 * |           |  |           |  |           |
 * |           |  |           |  |           |
 * -------------  |           |  -------------
 * -------------  |           |  -------------
 * |           |  |           |  |           |
 * |           |  |           |  |           |
 * |           |  |           |  |           |
 * |           |  |           |  |           |
 * -------------  -------------  -------------
 * This layout should function like side until
 * a third window is opened, after which
 * a third row should be created
*/



pub fn stack_sides(
    clients: &[&Client],
    _: Option<Xid>,
    monitor_region: &Region,
    _: u32,
    _: f32
) -> Vec<ResizeAction> {
    let n = clients.len() as u32;
    match n {
        0|1 => {
            monitor_region
                .as_columns(n)
                .iter()
                .zip(clients)
                .map(|(r,c)| (c.id(), Some(*r)))
                .collect()

        }
        2|3 => {
            let columns = monitor_region.as_columns(2);
            columns[0].as_rows(1)
                .into_iter()
                .chain(columns[1].as_rows(n.saturating_sub(1)))
                .zip(clients)
                .map(|(r,c)| (c.id(), Some(r)))
                .collect()   
        }
        _ => {
            let columns = monitor_region.as_columns(3);
            columns[1].as_rows(1)
                .into_iter()
                .chain(columns[2].as_rows(2))
                .chain(columns[0].as_rows(n-3))
                .zip(clients)
                .map(|(r,c)| (c.id(), Some(r)))
                .collect()
        }
    }
}
