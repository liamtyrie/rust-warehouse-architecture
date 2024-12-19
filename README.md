# Warehouse Architecture.

### Exploring how a large warehouse (eg Amazon) processes its products

The purpose of this project is to create an advanced event-driven architecture in Rust. Utilising the Transactional Outbox Pattern using the following technologies:

- Rust
- Apache Kafka
- MongoDB
- React

The project will consist of many services, running independently from each other, to create a fully decoupled system.

### Inbound Service

<img alt="inbound_service" width="650px" src="Inbound_Service.png" />
When products are delivered to the warehouse, these are scanned, which writes them to the database.

This then processes the incoming stock, writes to the outbox table and the inbound table.

Using Mongo Change Streams, we can wait for entries in the Outbox table, which will then create a message to send to the Inbound Shipments Outbox Service.

The inbound shipments outbox service will then send an event in the fulfillment request topic, stating a product is waiting to be checked and fulfilled.

Currently, this runs from a CSV instead of someone calling an API with a new product scan. This will eventually be implemented as a front-end app, where users can scan a QR code that contains all of the info of the product that has been delivered.

[<img alt="inbound_service" width="40px" src="Inbound_Service.png" />]
