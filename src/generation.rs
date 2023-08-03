use ascii_converter::decimals_to_string;
use rand::{Rng, seq::SliceRandom, thread_rng}; 
use serde::{Serialize, Deserialize};
use std::{fs, path::Path};
use crate::{
    categorize, 
    node::Node, 
    activation::ActivationFunction, 
    DEBUG, 
    error::DarjeelingError,
    input::Input, 
    types::{Types, Types::Boolean}
};

/// The top-level neural network struct
/// Sensor and answer represents which layer sensor and answer are on
#[derive(Debug, Serialize, Deserialize)]
pub struct NeuralNetwork {
    node_array: Vec<Vec<Node>>,
    sensor: Option<usize>,
    answer: Option<usize>,
    parameters: Option<u128>,
    activation_function: ActivationFunction
}
#[warn(clippy::unwrap_in_result)]
impl NeuralNetwork {
    
    /// Constructor function for the neural network
    /// Fills a Neural Network's node_array with empty nodes. 
    /// Initializes random starting link and bias weights between -.5 and .5
    /// 
    /// ## Params
    /// - Inputs: The number of sensors in the input layer
    /// - Hidden: The number of hidden nodes in each layer
    /// - Answer: The number of answer nodes, or possible categories
    /// - Hidden Layers: The number of different hidden layers
    /// 
    /// ## Examples
    /// ``` rust
    /// use darjeeling::{
    ///     activation::ActivationFunction,
    ///     categorize::NeuralNetwork
    /// };
    /// 
    /// let inputs: i32 = 10;
    /// let hidden: i32 = 40;
    /// let answer: i32 = 2;
    /// let hidden_layers: i32 = 1;
    /// let mut net: NeuralNetwork = NeuralNetwork::new(inputs, hidden, answer, hidden_layers, ActivationFunction::Sigmoid);
    /// ```
    pub fn new(pixel_input: i32, hidden_num: i32, pixel_output: i32, hidden_layers: i32, activation_function: ActivationFunction) -> NeuralNetwork {
        let mut net: NeuralNetwork = NeuralNetwork { node_array: vec![], sensor: Some(0), answer: Some(hidden_layers as usize + 1), parameters: None, activation_function};
        let mut rng = rand::thread_rng();
        net.node_array.push(vec![]);    
        for _i in 0..pixel_input {
            net.node_array[net.sensor.unwrap()].push(Node::new(&vec![], None));
        }

        for i in 1..hidden_layers + 1 {
            let mut hidden_vec:Vec<Node> = vec![];
            let hidden_links = net.node_array[(i - 1) as usize].len();
            if DEBUG { println!("Hidden Links: {:?}", hidden_links) }
            for _j in 0..hidden_num{
                hidden_vec.push(Node { link_weights: vec![], link_vals: vec![], links: hidden_links, err_sig: None, correct_answer: None, cached_output: None, category: None, b_weight: None });
            }
            net.node_array.push(hidden_vec);
        }

        net.node_array.push(vec![]);
        let answer_links = net.node_array[hidden_layers as usize].len();
        println!("Answer Links: {:?}", answer_links);
        for _i in 0..pixel_output {
            net.node_array[net.answer.unwrap()].push(Node { link_weights: vec![], link_vals: vec![], links: answer_links, err_sig: None, correct_answer: None, cached_output: Some(0.0), category: None, b_weight: None });
        }
        
        for layer in &mut net.node_array{
            for node in layer{
                node.b_weight = Some(rng.gen_range(-0.5..0.5));
                if DEBUG { println!("Made it to pushing link weights") }
                for _i in 0..node.links {
                    node.link_weights.push(rng.gen_range(-0.5..0.5));
                    node.link_vals.push(None);
                }
            }
        }
        let mut params = 0;
        for i in 0..net.node_array.len() {
            for j in 0..net.node_array[i].len() {
                params += 1 + net.node_array[i][j].links as u128;
            }
        }
        net.parameters = Some(params);
        net
    }

    /// Trains a neural model to generate new data formatted as inputs, based on the given data
    /// 
    /// ## Params
    /// - Data: List of inputs to be trained on
    /// - Learning Rate: The modifier that is applied to link weights as they're adjusted.
    /// Try fiddling with this one, but -1.5 - 1.5 is recommended to start.
    /// - Name: The model name
    /// - Max Cycles: The maximum number of epochs the training will run for.
    /// - Distinguising Learning Rate: The learning rate for the distinguishing model.
    /// - Distinguishing Hidden Neurons: The number of hidden neurons in each layer of the distinguishing model.
    /// - Distinguising Hidden Layers: The number of hidden layers in the distinguishing model.
    /// - Distinguishing Activation: The activation function of the distinguishing model.
    /// 
    /// ## Returns
    /// The falable name of the model that this neural network trained
    /// 
    /// ## Err
    /// - ### WriteModelFailed
    /// There was a problem when saving the model to a file
    /// 
    /// - ### ModelNameAlreadyExists
    /// The random model name chosen already exists
    /// Change the name or retrain
    /// 
    /// - ### RemoveModelFailed
    /// Everytime a new distinguishing model is written to the project folder, the previous one has to be removed.
    /// This removal failed,
    /// 
    /// - ### DistinguishingModel 
    /// The distinguishing model training failed.
    /// 
    /// - ### UnknownError
    /// Not sure what happened, but something failed
    /// 
    /// Make an issue on the [darjeeling](https://github.com/Ewie21/darjeeling) github page
    /// Or contact me at elocolburn@comcast.net
    /// 
    /// ## TODO: Refactor to pass around the neural net, not the model name
    /// 
    /// ## Examples
    /// ```ignore
    /// use darjeeling::{
    ///     generation::NeuralNetwork,
    ///     activation::ActivationFunction,
    ///     input::Input, 
    ///     // This file may not be avaliable
    ///     // Everything found here will be hyper-specific to your project.
    ///     tests::{categories_str_format, file}
    /// };
    /// 
    /// // A file with data
    /// // To make sure the networked is properly trained, make sure it follows some sort of pattern
    /// // This is just sample data, for accurate results, around 3800 datapoints is needed
    /// // 1 2 3 4 5 6 7 8
    /// // 3 2 5 4 7 6 1 8
    /// // 0 2 5 4 3 6 1 8
    /// // 7 2 3 4 9 6 1 8
    /// // You also need to write the file input function
    /// // Automatic file reading and formatting function coming soon
    /// let mut data: Vec<Input> = file();
    /// let mut net = NeuralNetwork::new(2, 2, 2, 1, ActivationFunction::Sigmoid);
    /// let model_name: String = net.learn(&mut data, 0.5, "gen", 100, 0.5, 10, 1, ActivationFunction::Sigmoid).unwrap();
    /// let new_data: Vec<Input> = net.test(data).unwrap();
    /// ```
    pub fn learn(
        &mut self, 
        data: &mut Vec<Input>, 
        learning_rate: f32, 
        name: &str, max_cycles: i32, 
        distinguising_learning_rate: f32, distinguising_hidden_neurons: i32,
        distinguising_hidden_layers: i32, distinguising_activation: ActivationFunction
    ) -> Result<String, DarjeelingError> {
        let mut epochs: f32 = 0.0;
        let hidden_layers = self.node_array.len() - 2;
        let mut model_name: Option<String> = None;
        let mut outputs: Vec<Input> = vec![];
        for _i in 0..max_cycles {
            #[allow(unused_assignments)]
            let mut mse = 0.0; // mse is for a single epoch
            data.shuffle(&mut thread_rng());
            for line in 0..data.len() {
                if DEBUG { println!("Training Checkpoint One Passed") }
                self.push_downstream(data, line as i32);
                let mut output = vec![];
                for i in 0..self.node_array[self.answer.unwrap()].len() {
                    output.push(self.node_array[self.answer.unwrap()][i].output(&self.activation_function));
                }
                outputs.push(Input::new(output, Some(Boolean(false)))); // false indicates not real data
                data[line].answer = Some(Boolean(true));
                outputs.push(data[line].clone());
            }
            if model_name.is_some() {
                let mut new_model = categorize::NeuralNetwork::read_model(model_name.clone().unwrap()).unwrap();
                match std::fs::remove_file(model_name.unwrap()) {
                    Ok(_) => {
                        model_name = match new_model.learn(
                            &mut outputs, 
                            vec![Boolean(true), Boolean(false)], 
                            distinguising_learning_rate, &("distinguishing".to_owned() + &name)) 
                            {
                                Ok((name, _err_percent, errmse)) => { mse = errmse; Some(name) },
                                Err(error) => return Err(DarjeelingError::DisinguishingModelError(error.to_string()))
                            };
                    },
                    Err(err) => return Err(DarjeelingError::RemoveModelFailed(err.to_string()))
                };
            } else {
                let mut new_model = categorize::NeuralNetwork::new(self.node_array[self.answer.unwrap()].len() as i32, distinguising_hidden_neurons, 2, distinguising_hidden_layers, distinguising_activation);
                match std::fs::remove_file(model_name.unwrap()) {
                    Ok(_) => {
                        model_name = match new_model.learn(
                            data, 
                            vec![Boolean(true), Boolean(false)], 
                            distinguising_learning_rate, 
                            &("distinguishing".to_owned() + &name)) 
                            {
                                Ok((name, _err_percent, errmse)) => { mse = errmse; Some(name) },
                                Err(error) => return Err(DarjeelingError::DisinguishingModelError(error.to_string()))
                            };
                    },
                    Err(err) => return Err(DarjeelingError::RemoveModelFailed(err.to_string()))
                };
                
            }
            
            self.backpropogate(learning_rate, hidden_layers as i32, mse);

            epochs += 1.0;
            println!("Epoch: {:?}", epochs);
        }
        #[allow(unused_mut)]
        let mut model_name: String;
        match self.write_model(&name) {
            Ok(m_name) => {
                model_name = m_name;
            },
            Err(error) => return Err(error)
        }
        Ok(model_name)
    }

    pub fn test(&mut self, data: &mut Vec<Input>) -> Result<Vec<Input>, DarjeelingError> {
        data.shuffle(&mut thread_rng());
        let mut outputs: Vec<Input> = vec![];
        for i in 0..data.len() {
            self.push_downstream(data, i as i32);
            let mut output = vec![];
            for i in 0..self.node_array[self.answer.unwrap()].len() {
                output.push(self.node_array[self.answer.unwrap()][i].output(&self.activation_function));
            }
            outputs.push(Input::new(output, None)); // false indicates not real data
        }
        Ok(outputs)
    }

    /// Passes in data to the sensors, pushs data 'downstream' through the network
    fn push_downstream(&mut self, data: &mut Vec<Input>, line: i32) {

        // Passes in data for input layer
        for i in 0..self.node_array[self.sensor.unwrap()].len() {
            let input  = data[line as usize].inputs[i];

            self.node_array[self.sensor.unwrap()][i].cached_output = Some(input);
        }

        // Feed-forward values for hidden and output layers
        for layer in 1..self.node_array.len() {

            for node in 0..self.node_array[layer].len() {

                for prev_node in 0..self.node_array[layer-1].len() {
                    
                    // self.node_array[layer][node].link_vals.push(self.node_array[layer-1][prev_node].cached_output.unwrap());
                    self.node_array[layer][node].link_vals[prev_node] = Some(self.node_array[layer-1][prev_node].cached_output.unwrap());
                    // I think this line needs to be un-commented
                    self.node_array[layer][node].output(&self.activation_function);
                    if DEBUG { if layer == self.answer.unwrap() { println!("Ran output on answer {:?}", self.node_array[layer][node].cached_output) } }
                }
                self.node_array[layer][node].output(&self.activation_function);
            }
        }
    }

    /// Finds the index and the brightest node in an array and returns it
    fn largest_node(&self) -> usize {
        let mut largest_node = 0;
        for node in 0..self.node_array[self.answer.unwrap()].len() {
            if self.node_array[self.answer.unwrap()][node].cached_output > self.node_array[self.answer.unwrap()][largest_node].cached_output {
                largest_node = node;
            }
        }
        largest_node
    }
    /// Goes back through the network adjusting the weights of the all the neurons based on their error signal
    fn backpropogate(&mut self, learning_rate: f32, hidden_layers: i32, mse: f32) {
        for answer in 0..self.node_array[self.answer.unwrap()].len() {
            if DEBUG { println!("Node: {:?}", self.node_array[self.answer.unwrap()][answer]); }
            self.node_array[self.answer.unwrap()][answer].compute_answer_err_sig_gen(mse);
            if DEBUG { println!("Error: {:?}", self.node_array[self.answer.unwrap()][answer].err_sig.unwrap()) }
        }
        self.adjust_hidden_weights(learning_rate, hidden_layers);
        // Adjusts weights for answer neurons
        for answer in 0..self.node_array[self.answer.unwrap()].len() {
            self.node_array[self.answer.unwrap()][answer].adjust_weights(learning_rate);
        }
    }

    #[allow(non_snake_case)]
    /// Adjusts the weights of all the hidden neurons in a network
    fn adjust_hidden_weights(&mut self, learning_rate: f32, hidden_layers: i32) {
        // HIDDEN represents the layer, while hidden represents the node of the layer
        for HIDDEN in 1..(hidden_layers + 1) as usize {            
            for hidden in 0..self.node_array[HIDDEN].len() {
                self.node_array[HIDDEN][hidden].err_sig = Some(0.0);
                for next_layer in 0..self.node_array[HIDDEN + 1 ].len() {
                    let next_weight = self.node_array[HIDDEN + 1][next_layer].link_weights[hidden];
                    self.node_array[HIDDEN + 1][next_layer].err_sig = match self.node_array[HIDDEN + 1][next_layer].err_sig.is_none() {
                        true => {
                            Some(0.0)
                        }, 
                        false => {
                            self.node_array[HIDDEN + 1][next_layer].err_sig
                        }
                    };
                    // This changes based on the activation function
                    self.node_array[HIDDEN][hidden].err_sig = Some(self.node_array[HIDDEN][hidden].err_sig.unwrap() + (self.node_array[HIDDEN + 1][next_layer].err_sig.unwrap() * next_weight));
                    if DEBUG { 
                        println!("next err sig {:?}", self.node_array[HIDDEN + 1][next_layer].err_sig.unwrap());
                        println!("next weight {:?}", next_weight);
                    }
                }
                let hidden_result = self.node_array[HIDDEN][hidden].cached_output.unwrap();
                let multiplied_value = self.node_array[HIDDEN][hidden].err_sig.unwrap() * (hidden_result) * (1.0 - hidden_result);
                if DEBUG { println!("new hidden errsig multiply: {:?}", multiplied_value); }
                self.node_array[HIDDEN][hidden].err_sig = Some(multiplied_value);

                if DEBUG { 
                    println!("\nLayer: {:?}", HIDDEN);
                    println!("Node: {:?}", hidden) 
                }

                self.node_array[HIDDEN][hidden].adjust_weights(learning_rate);
            }
        }
    }

    /// Not needed for now
    /// Analyses the chosen answer node's result.
    /// Also increments sum and count
    /// Err if string requested and float exceeds u8 limit (fix by parsing the floats and slicing them)
    fn self_analysis<'b>(&'b self, epochs: &mut Option<f32>, sum: &'b mut f32, count: &'b mut f32, data: &mut Vec<Input>, line: usize, expected_type: Types) -> Result<Vec<Types>, DarjeelingError> {
        // println!("answer {}", self.answer.unwrap());
        // println!("largest index {}", self.largest_node());
        // println!("{:?}", self);
        let brightest_node: &Node = &self.node_array[self.answer.unwrap()][self.largest_node()];
        let brightness: f32 = brightest_node.cached_output.unwrap();

        if !(epochs.is_none()) { // This lets us use the same function for testing and training     
            if epochs.unwrap() % 10.0 == 0.0 && epochs.unwrap() != 0.0 {
                println!("\n-------------------------\n");
                println!("Epoch: {:?}", epochs);
                println!("Category: {:?} \nBrightness: {:?}", brightest_node.category.as_ref().unwrap(), brightness);
                if DEBUG {
                    let dimest_node: &Node = &self.node_array[self.answer.unwrap()][self.node_array[self.answer.unwrap()].len()-1-self.largest_node()];
                    println!("Chosen category: {:?} \nDimest Brightness: {:?}", dimest_node.category.as_ref().unwrap(), dimest_node.cached_output.unwrap());
                }
            }
        }

        if DEBUG { println!("Category: {:?} \nBrightness: {:?}", brightest_node.category.as_ref().unwrap(), brightness); }
        if brightest_node.category.as_ref().unwrap().eq(&data[line].answer.as_ref().unwrap()) { 
            if DEBUG { println!("Correct Answer Chosen"); }
            if DEBUG { println!("Sum++"); }
            *sum += 1.0;
        }
        *count += 1.0;
        let mut ret: Vec<Types> = vec![];
        match expected_type {
            Types::Integer(_) => {
                for node in &self.node_array[self.answer.unwrap()] {
                    let int = node.cached_output.unwrap() as i32;
                    ret.push(Types::Integer(int));
                }
            }
            Types::Boolean(_) => {
                for node in &self.node_array[self.answer.unwrap()] {
                    let bool = node.cached_output.unwrap() > 0.0;
                    ret.push(Types::Boolean(bool));
                }
            }
            Types::Float(_) => {
                for node in &self.node_array[self.answer.unwrap()] {
                    ret.push(Types::Float(node.cached_output.unwrap()));
                }
            }
            Types::String(_) => {
                for node in &self.node_array[self.answer.unwrap()] {
                    let inputs = vec![(node.cached_output.unwrap() as u8)];
                    let buff = match decimals_to_string(&inputs) {
                        Ok(val) => val,
                        Err(err) => return Err(DarjeelingError::SelfAnalysisStringConversion(err))
                    };
                    ret.push(Types::String(buff));
                }
            }

        };
        Ok(ret)
    }

    /// Serializes a trained model as a .darj file so it can be used later
    /// 
    /// ## Returns
    /// The name of the model
    /// 
    /// ## Error
    /// ### WriteModelFailed:
    /// Writing to the file failed
    /// 
    /// Wraps the models name
    /// ### UnknownError:
    /// Something else went wrong
    /// 
    /// Wraps error
    pub fn write_model(&mut self, name: &str) -> Result<String, DarjeelingError> {
        let mut rng = rand::thread_rng();
        let file_num: u32 = rng.gen();
        let model_name: String = format!("model_{}_{}.darj", name, file_num);

        match Path::new(&model_name).try_exists() {

            Ok(false) => {
                let _file: fs::File = fs::File::create(&model_name).unwrap();
                let mut serialized = "".to_string();
                println!("write, length: {}", self.node_array.len());
                for i in 0..self.node_array.len() {
                    if i != 0 {
                        let _ = serialized.push_str("lb\n");
                    }
                    for j in 0..self.node_array[i].len() {
                        for k in 0..self.node_array[i][j].link_weights.len() {
                            print!("{}", self.node_array[i][j].link_weights[k]);
                            if k == self.node_array[i][j].link_weights.len() - 1 {
                                let _ = serialized.push_str(format!("{}", self.node_array[i][j].link_weights[k]).as_str());
                            } else {
                                let _ = serialized.push_str(format!("{},", self.node_array[i][j].link_weights[k]).as_str());
                            }                        
                        }
                        let _ = serialized.push_str(format!(";{}", self.node_array[i][j].b_weight.unwrap().to_string()).as_str()); 
                        let _ = serialized.push_str("\n");
                    }
                }
                serialized.push_str("lb\n");                    
                serialized.push_str(format!("{}", self.activation_function).as_str());
                // println!("Serialized: {:?}", serialized);
                match fs::write(&model_name, serialized) {
                    Ok(()) => {
                        println!("Model {:?} Saved", file_num);
                        Ok(model_name)
                    },
                    Err(_error) => {
                        Err(DarjeelingError::WriteModelFailed(model_name))
                    }
                }
            },
            Ok(true) => {
                return self.write_model(name);
            },
            Err(error) => Err(DarjeelingError::UnknownError(error.to_string()))
        }
    }

    /// Reads a serizalized Neural Network
    /// 
    /// ## Params
    /// - Model Name: The name(or more helpfully the path) of the model to be read
    /// 
    /// ## Returns
    /// A neural network read from a serialized .darj file
    /// 
    /// ## Err
    /// If the file cannnot be read, or if the file does not contain a valid serialized Neural Network
    pub fn read_model(model_name: String) -> Result<NeuralNetwork, DarjeelingError> {

        println!("Loading model");
        
        // Err if the file reading fails
        let serialized_net: String = match fs::read_to_string(&model_name) {
            
            Ok(serizalized_net) => serizalized_net,
            Err(error) => return Err(DarjeelingError::ReadModelFailed(model_name.clone() + ";" +  &error.to_string()))
        };
 
        let mut node_array: Vec<Vec<Node>> = vec![];
        let mut layer: Vec<Node> = vec![];
        let mut activation: Option<ActivationFunction> = None;
        for i in serialized_net.lines() {
            match i {
                "sigmoid" => activation = Some(ActivationFunction::Sigmoid),

                "linear" => activation = Some(ActivationFunction::Linear),

                "tanh" => activation = Some(ActivationFunction::Tanh),
 
                "step" => activation = Some(ActivationFunction::Step),

                _ => {
                
                    if i.trim() == "lb" {
                        node_array.push(layer.clone());
                        // println!("pushed layer {:?}", layer.clone());
                        layer = vec![];
                        continue;
                    }
                    #[allow(unused_mut)]
                    let mut node: Option<Node>;
                    if node_array.len() == 0 {
                        let b_weight: Vec<&str> = i.split(";").collect();
                        // println!("b_weight: {:?}", b_weight);
                        node = Some(Node::new(&vec![], Some(b_weight[1].parse().unwrap())));
                    } else {
                        let node_data: Vec<&str> = i.trim().split(";").collect();
                        let str_weight_array: Vec<&str> = node_data[0].split(",").collect();
                        let mut weight_array: Vec<f32> = vec![];
                        let b_weight: &str = node_data[1];
                        // println!("node_data: {:?}", node_data);
                        // println!("array {:?}", str_weight_array);
                        for weight in 0..str_weight_array.len() {
                            // println!("testing here {:?}", str_weight_array[weight]);
                            let val: f32 = str_weight_array[weight].parse().unwrap();
                            weight_array.push(val);
                        }
                        // print!("{}", b_weight);
                        node = Some(Node::new(&weight_array, Some(b_weight.parse().unwrap())));
                    }
                    
                    layer.push(node.expect("Both cases provide a Some value for node"));
                    // println!("layer: {:?}", layer.clone())
                }
            }
            
        }
        // println!("node array size {}", node_array.len());
        let sensor: Option<usize> = Some(0);
        let answer: Option<usize> = Some(node_array.len() - 1);
        
        let net = NeuralNetwork {
            node_array,
            sensor,
            answer,
            parameters: None,
            activation_function: activation.unwrap()
        };
        // println!("node array {:?}", net.node_array);

        Ok(net)
    }
}

