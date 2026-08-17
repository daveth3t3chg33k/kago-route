//! Embedded demo flow (identical to `docs/examples/farmer-order.yaml`) used
//! when no `FLOW_SCHEMA_PATH` is configured, so the engine is runnable with
//! zero setup.

pub const FARMER_ORDER: &str = r#"
schema: "kagoroute/menu/1.0"

flow:
  id: "farmer-order"
  name: "Farmer Supply Order"
  description: "Order seed & fertilizer from Tuma Farm Supplies, paid via M-Pesa."
  version: 4
  start: "welcome"
  timeouts:
    session: 120
    step: 20
  webhooks:
    onComplete:
      url: "https://api.tumafarms.co.ke/v1/webhooks/ussd/complete"
      secret: "whsec_tuma_demo"
      events: ["complete", "payment.result"]
  variables:
    - name: "product"
    - name: "unitPrice"
      type: "int"
    - name: "qty"
      type: "int"
    - name: "total"
      type: "int"

  nodes:
    welcome:
      type: menu
      text: "Tuma Farm Supplies\n1. Order inputs\n0. Exit"
      options:
        "1":
          - goto: "product"
        "0":
          - goto: "farewell"

    product:
      type: menu
      text: "Select product:\n1. Maize seed (KES 3,500/bag)\n2. Fertilizer (KES 2,200/bag)"
      options:
        "1":
          - set: { product: "maize-seed", unitPrice: 3500 }
            goto: "qty"
        "2":
          - set: { product: "fertilizer", unitPrice: 2200 }
            goto: "qty"

    qty:
      type: input
      prompt: "How many bags of {product}? (1-50)"
      variable: "qty"
      validate:
        type: int
        min: 1
        max: 50
      onInvalid:
        text: "Enter a whole number between 1 and 50."
        goto: "qty"
      next: "totals"

    totals:
      type: action
      compute:
        total: "unitPrice * qty"
      next: "confirm"

    confirm:
      type: menu
      text: "Order: {qty} x {product} = KES {total}\n1. Pay via M-Pesa\n2. Cancel"
      options:
        "1":
          - when: { var: "total", op: "gte", value: 10000 }
            goto: "stk_flagged"
          - goto: "stk_standard"
        "2":
          - goto: "welcome"

    stk_standard:
      type: end
      text: "A payment request has been sent to your phone.\nEnter your M-Pesa PIN to confirm."
      payments:
        mpesa:
          shortCode: "483242"
          amountExpr: "total"
          phoneExpr: "$phone"
          accountRef: "TUMA-{qty}"
          transactionDesc: "Farm inputs"

    stk_flagged:
      type: end
      text: "A payment request has been sent to your phone.\nAn agent will call to confirm large orders."
      payments:
        mpesa:
          shortCode: "483242"
          amountExpr: "total"
          phoneExpr: "$phone"
          accountRef: "TMAF-{qty}"
          transactionDesc: "Farm flagged"

    farewell:
      type: end
      text: "Thank you. Dial *483*42# anytime to order."
"#;
