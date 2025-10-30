/**
 * Transform client history template with date range calculations
 * @param {Object} template - The JSON template
 * @param {Object} params - URL parameters (cliente_id, fecha_desde, fecha_hasta)
 * @param {Object} context - Request context (timestamp, requestId)
 * @returns {Object} Transformed JSON response
 */
function transform(template, params, context) {
    // Calculate days in the period
    const fechaDesde = new Date(params.fecha_desde);
    const fechaHasta = new Date(params.fecha_hasta);
    const diffTime = Math.abs(fechaHasta - fechaDesde);
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    
    template.periodo.dias_consultados = diffDays;
    
    // Generate dynamic client information based on client ID
    const clientNumber = parseInt(params.cliente_id.substring(1)); // Remove 'C' prefix
    const clientTypes = ["regular", "premium", "nuevo", "esporádico"];
    const clientNames = [
        "Tienda La Esquina", "Supermercado Don Luis", "Abarrotes El Buen Precio",
        "Minisuper Central", "Comercial Familiar", "Tienda de la Colonia"
    ];
    
    template.informacion_cliente.nombre = clientNames[clientNumber % clientNames.length];
    template.informacion_cliente.tipo_cliente = clientTypes[clientNumber % clientTypes.length];
    
    // Generate purchase history based on the date range
    template.historial_compras = [];
    let totalAmount = 0;
    let totalOrders = 0;
    const products = ["355ML", "600ML", "1L", "2L", "SNACKS_FAMILIARES", "PREMIUM_500ML"];
    
    // Generate 1-5 purchases in the period
    const numPurchases = Math.min(diffDays, Math.floor(Math.random() * 5) + 1);
    
    for (let i = 0; i < numPurchases; i++) {
        // Generate random date within the range
        const randomTime = fechaDesde.getTime() + Math.random() * (fechaHasta.getTime() - fechaDesde.getTime());
        const purchaseDate = new Date(randomTime);
        const dateStr = purchaseDate.toISOString().split('T')[0];
        
        // Generate random items
        const numItems = Math.floor(Math.random() * 3) + 1;
        const items = [];
        let orderTotal = 0;
        
        for (let j = 0; j < numItems; j++) {
            const sku = products[Math.floor(Math.random() * products.length)];
            const quantity = (Math.floor(Math.random() * 3) + 1) * 12; // 12, 24, 36
            const unitPrice = 15.50 + Math.random() * 20; // 15.50 - 35.50
            const subtotal = quantity * unitPrice;
            
            items.push({
                sku: sku,
                cantidad: quantity,
                precio_unitario: Math.round(unitPrice * 100) / 100,
                subtotal: Math.round(subtotal * 100) / 100
            });
            
            orderTotal += subtotal;
        }
        
        template.historial_compras.push({
            fecha: dateStr,
            pedido_id: "PED-" + String(i + 1).padStart(3, '0'),
            items: items,
            total: Math.round(orderTotal * 100) / 100,
            forma_pago: Math.random() > 0.6 ? "credito" : "contado",
            asesor: "A-" + (77 + (clientNumber % 5))
        });
        
        totalAmount += orderTotal;
        totalOrders++;
    }
    
    // Update summary
    template.resumen_periodo.total_pedidos = totalOrders;
    template.resumen_periodo.monto_total = Math.round(totalAmount * 100) / 100;
    template.resumen_periodo.promedio_por_pedido = totalOrders > 0 ? 
        Math.round((totalAmount / totalOrders) * 100) / 100 : 0;
    
    // Calculate most purchased products
    const productCounts = {};
    template.historial_compras.forEach(purchase => {
        purchase.items.forEach(item => {
            productCounts[item.sku] = (productCounts[item.sku] || 0) + item.cantidad;
        });
    });
    
    const sortedProducts = Object.entries(productCounts)
        .sort(([,a], [,b]) => b - a)
        .map(([product]) => product);
    
    template.resumen_periodo.productos_mas_comprados = sortedProducts.slice(0, 3);
    template.resumen_periodo.frecuencia_compra_dias = totalOrders > 1 ? 
        Math.round((diffDays / totalOrders) * 10) / 10 : diffDays;
    
    // Generate trends based on client type and purchase pattern
    const isGrowthClient = template.informacion_cliente.tipo_cliente === "premium" || 
                          totalAmount > 2000;
    
    template.tendencias.crecimiento_vs_periodo_anterior = isGrowthClient ? 
        "+" + (Math.random() * 20 + 5).toFixed(1) + "%" :
        (Math.random() * 30 - 15).toFixed(1) + "%";
    
    // Products trending up/down
    template.tendencias.productos_en_alza = sortedProducts.slice(0, 2);
    template.tendencias.productos_en_baja = sortedProducts.length > 3 ? 
        sortedProducts.slice(-1) : [];
    
    // Loyalty score based on purchase frequency and amount
    const frequencyScore = Math.min(100, (totalOrders / diffDays) * 100 * 30);
    const amountScore = Math.min(100, totalAmount / 50);
    template.tendencias.score_fidelidad = Math.round((frequencyScore + amountScore) / 2);
    
    // Update metadata
    template.metadata.generado_en = context.timestamp;
    template.metadata.total_registros = totalOrders;
    template.metadata.request_id = context.requestId;
    template.metadata.parametros_procesados = {
        cliente_id: params.cliente_id,
        fecha_desde: params.fecha_desde,
        fecha_hasta: params.fecha_hasta,
        dias_periodo: diffDays,
        tipo_cliente_detectado: template.informacion_cliente.tipo_cliente
    };
    
    return template;
}