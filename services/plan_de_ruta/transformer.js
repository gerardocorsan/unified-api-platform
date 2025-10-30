/**
 * Transform template with dynamic parameters
 * @param {Object} template - The JSON template
 * @param {Object} params - URL parameters (ruta_id, fecha)
 * @param {Object} context - Request context (timestamp, requestId)
 * @returns {Object} Transformed JSON response
 */
function transform(template, params, context) {
    // Set current timestamp
    template.metadata.generado_en = context.timestamp;
    
    // Customize recommendations based on route and date
    const routeType = params.ruta_id.charAt(0); // P = Premium, R = Regular, etc.
    const dayOfWeek = new Date(params.fecha).getDay(); // 0=Sunday, 1=Monday, etc.
    
    // Modify recommendations based on route type
    if (routeType === 'P') {
        // Premium routes get more high-priority recommendations
        template.clientes_en_ruta.forEach(cliente => {
            cliente.recomendaciones.forEach(rec => {
                if (rec.prioridad === 'media') {
                    rec.prioridad = 'alta';
                }
                // Add route-specific recommendation IDs
                rec.recomendacion_id = rec.recomendacion_id + '-' + params.ruta_id;
            });
        });
        
        template.resumen_ruta.prioridad_alta += 2;
        template.resumen_ruta.prioridad_media -= 2;
        template.resumen_ruta.potencial_venta_total *= 1.25; // 25% increase for premium routes
    }
    
    // Modify recommendations based on day of week
    if (dayOfWeek === 1) { // Monday - stock alerts are more important
        template.clientes_en_ruta.forEach(cliente => {
            cliente.recomendaciones.forEach(rec => {
                if (rec.tipo === 'ALERTA_QUIEBRE_STOCK') {
                    rec.prioridad = 'critica';
                    rec.payload.urgencia_dia = 'lunes_critico';
                }
            });
        });
    } else if (dayOfWeek === 5) { // Friday - focus on offers
        template.clientes_en_ruta.forEach(cliente => {
            cliente.recomendaciones.forEach(rec => {
                if (rec.tipo === 'OFERTA_DINAMICA') {
                    rec.payload.descuento_porcentaje = rec.payload.descuento_porcentaje * 1.5;
                    rec.payload.urgencia_dia = 'viernes_especial';
                }
            });
        });
    }
    
    // Add date-specific insights
    const fechaObj = new Date(params.fecha);
    const dayOfMonth = fechaObj.getDate();
    
    // End of month - focus on volume recovery
    if (dayOfMonth > 25) {
        template.clientes_en_ruta.forEach(cliente => {
            // Add volume recovery recommendation if not present
            const hasVolumeRec = cliente.recomendaciones.some(r => r.tipo === 'RECUPERACION_VOLUMEN');
            if (!hasVolumeRec) {
                cliente.recomendaciones.push({
                    "recomendacion_id": "rec-vol-" + cliente.cliente_id,
                    "tipo": "RECUPERACION_VOLUMEN",
                    "titulo_accion": "Impulso fin de mes",
                    "prioridad": "alta",
                    "payload": {
                        "objetivo_mes": "Alcanzar meta mensual",
                        "incentivo_disponible": "Descuento 10% compras mayores a $5000",
                        "dias_restantes": 31 - dayOfMonth
                    },
                    "feedback_config": {
                        "opciones": ["Interesado", "Pedirá cotización", "No aplicable", "Pospondrá"],
                        "comentario_habilitado": true
                    }
                });
            }
        });
        
        template.resumen_ruta.total_recomendaciones += template.clientes_en_ruta.length;
        template.resumen_ruta.enfoque_especial = "fin_de_mes";
    }
    
    // Update visit time based on number of recommendations
    const totalRecs = template.resumen_ruta.total_recomendaciones;
    const estimatedHours = Math.max(2.0, totalRecs * 0.15 + 1.5);
    template.resumen_ruta.tiempo_estimado_visitas = estimatedHours.toFixed(1) + " horas";
    
    // Add route analytics
    template.analytics = {
        "tipo_ruta": routeType === 'P' ? 'Premium' : 'Estándar',
        "dia_semana": ['Domingo', 'Lunes', 'Martes', 'Miércoles', 'Jueves', 'Viernes', 'Sábado'][dayOfWeek],
        "factores_aplicados": [],
        "score_optimizacion": Math.floor(Math.random() * 20) + 80 // 80-99
    };
    
    if (routeType === 'P') {
        template.analytics.factores_aplicados.push("premium_route_boost");
    }
    
    if (dayOfWeek === 1) {
        template.analytics.factores_aplicados.push("monday_stock_priority");
    } else if (dayOfWeek === 5) {
        template.analytics.factores_aplicados.push("friday_offer_boost");
    }
    
    if (dayOfMonth > 25) {
        template.analytics.factores_aplicados.push("end_of_month_volume");
    }
    
    // Add request tracking
    template.metadata.request_id = context.requestId;
    template.metadata.parametros_procesados = {
        "ruta_id": params.ruta_id,
        "fecha": params.fecha,
        "tipo_ruta_detectado": template.analytics.tipo_ruta,
        "dia_semana_detectado": template.analytics.dia_semana
    };
    
    return template;
}